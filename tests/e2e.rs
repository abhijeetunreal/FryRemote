//! End-to-end test: the real client code against an in-process mock TV that
//! speaks TLS with client certificates and both TV protocols.

use rustls::crypto::{ring, verify_tls12_signature, verify_tls13_signature, CryptoProvider};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, UnixTime};
use rustls::server::danger::{ClientCertVerified, ClientCertVerifier};
use rustls::{DigitallySignedStruct, DistinguishedName, ServerConfig, SignatureScheme};
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_rustls::TlsAcceptor;
use tvremote::creds::{rsa_modulus_exponent, Creds};
use tvremote::pairing::{CodeError, PairingSession};
use tvremote::proto::*;
use tvremote::state::{ConnStatus, Shared};
use tvremote::wire::{read_msg, write_msg};

#[derive(Debug)]
struct AcceptAnyClient(Arc<CryptoProvider>);

impl ClientCertVerifier for AcceptAnyClient {
    fn root_hint_subjects(&self) -> &[DistinguishedName] {
        &[]
    }
    fn verify_client_cert(
        &self,
        _e: &CertificateDer<'_>,
        _i: &[CertificateDer<'_>],
        _n: UnixTime,
    ) -> Result<ClientCertVerified, rustls::Error> {
        Ok(ClientCertVerified::assertion())
    }
    fn verify_tls12_signature(
        &self,
        m: &[u8],
        c: &CertificateDer<'_>,
        d: &DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        verify_tls12_signature(m, c, d, &self.0.signature_verification_algorithms)
    }
    fn verify_tls13_signature(
        &self,
        m: &[u8],
        c: &CertificateDer<'_>,
        d: &DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        verify_tls13_signature(m, c, d, &self.0.signature_verification_algorithms)
    }
    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.0.signature_verification_algorithms.supported_schemes()
    }
}

fn acceptor(tv: &Creds) -> TlsAcceptor {
    let provider = Arc::new(ring::default_provider());
    let cfg = ServerConfig::builder_with_provider(provider.clone())
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_client_cert_verifier(Arc::new(AcceptAnyClient(provider)))
        .with_single_cert(
            vec![CertificateDer::from(tv.cert_der.clone())],
            PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(tv.key_der.clone())),
        )
        .unwrap();
    TlsAcceptor::from(Arc::new(cfg))
}

fn status_ok() -> PairingMessage {
    PairingMessage { protocol_version: 2, status: 200, ..Default::default() }
}

/// Mock TV pairing endpoint. Returns (port, the 6-char code it "displays").
async fn mock_pairing_tv(tv: Arc<Creds>) -> (u16, tokio::sync::oneshot::Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let acc = acceptor(&tv);

    // Pre-compute the code we will display: needs the client's key, which we
    // only learn at handshake time, so compute it inside the task and publish it.
    let (code_tx, code_rx) = tokio::sync::oneshot::channel::<String>();
    tokio::spawn(async move {
        let (tcp, _) = listener.accept().await.unwrap();
        let mut s = acc.accept(tcp).await.unwrap();
        let client_cert = s.get_ref().1.peer_certificates().unwrap()[0].to_vec();
        let client_key = rsa_modulus_exponent(&client_cert).unwrap();
        let server_key = tv.modulus_exponent().unwrap();

        // choose code tail bytes, derive the check byte from the hash
        let tail = [0x3c, 0x7e];
        let mut h = Sha256::new();
        h.update(&client_key.0);
        h.update(&client_key.1);
        h.update(&server_key.0);
        h.update(&server_key.1);
        h.update(tail);
        let digest = h.finalize().to_vec();
        let code = hex::encode([digest[0], tail[0], tail[1]]);
        code_tx.send(code).unwrap();

        let req: PairingMessage = read_msg(&mut s).await.unwrap();
        assert!(req.pairing_request.is_some());
        let mut ack = status_ok();
        ack.pairing_request_ack = Some(PairingRequestAck { server_name: "Mock TV".into() });
        write_msg(&mut s, &ack).await.unwrap();

        let opts: PairingMessage = read_msg(&mut s).await.unwrap();
        assert_eq!(opts.options.unwrap().input_encodings[0].symbol_length, 6);
        let mut reply = status_ok();
        reply.options = Some(PairingOptions {
            input_encodings: vec![],
            output_encodings: vec![],
            preferred_role: 2,
        });
        write_msg(&mut s, &reply).await.unwrap();

        let cfg: PairingMessage = read_msg(&mut s).await.unwrap();
        assert!(cfg.configuration.is_some());
        let mut ack = status_ok();
        ack.configuration_ack = Some(PairingConfigurationAck {});
        write_msg(&mut s, &ack).await.unwrap();

        let sec: PairingMessage = read_msg(&mut s).await.unwrap();
        assert_eq!(sec.secret.unwrap().secret, digest, "client sent the wrong secret");
        let mut ack = status_ok();
        ack.secret_ack = Some(PairingSecretAck { secret: digest });
        write_msg(&mut s, &ack).await.unwrap();
    });
    (port, code_rx)
}

#[tokio::test]
async fn pairing_flow_with_wrong_then_right_code() {
    let tv = Arc::new(Creds::generate().unwrap());
    let client = Creds::generate().unwrap();
    let (port, code_rx) = mock_pairing_tv(tv).await;

    let mut session = PairingSession::start_on("127.0.0.1", port, &client, "test").await.unwrap();
    // the mock TV "displays" its code once the handshake has completed
    let code = code_rx.await.unwrap();

    // A code with a bad check byte is rejected locally, without touching the TV.
    let mut bad = hex::decode(&code).unwrap();
    bad[0] ^= 0xff;
    match session.finish(&hex::encode(bad)).await {
        Err(CodeError::WrongCode) => {}
        other => panic!("expected WrongCode, got {other:?}"),
    }

    session.finish(&code).await.expect("correct code should pair");
}

#[tokio::test]
async fn remote_session_keys_pings_and_launch() {
    let tv = Arc::new(Creds::generate().unwrap());
    let client = Arc::new(Creds::generate().unwrap());
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let acc = acceptor(&tv);

    let (seen_tx, mut seen_rx) = mpsc::unbounded_channel::<RemoteMessage>();
    tokio::spawn(async move {
        let (tcp, _) = listener.accept().await.unwrap();
        let mut s = acc.accept(tcp).await.unwrap();

        write_msg(&mut s, &RemoteMessage {
            configure: Some(RemoteConfigure { code1: 622, device_info: None }),
            ..Default::default()
        })
        .await
        .unwrap();
        let m: RemoteMessage = read_msg(&mut s).await.unwrap();
        assert_eq!(m.configure.unwrap().code1, 622);

        write_msg(&mut s, &RemoteMessage {
            set_active: Some(RemoteSetActive { active: 622 }),
            ..Default::default()
        })
        .await
        .unwrap();
        let m: RemoteMessage = read_msg(&mut s).await.unwrap();
        assert_eq!(m.set_active.unwrap().active, 622);

        write_msg(&mut s, &RemoteMessage {
            start: Some(RemoteStart { started: true }),
            set_volume_level: Some(RemoteSetVolumeLevel { volume_max: 100, volume_level: 37, volume_muted: false }),
            ..Default::default()
        })
        .await
        .unwrap();

        write_msg(&mut s, &RemoteMessage {
            ping_request: Some(RemotePingRequest { val1: 7, val2: 0 }),
            ..Default::default()
        })
        .await
        .unwrap();
        let m: RemoteMessage = read_msg(&mut s).await.unwrap();
        assert_eq!(m.ping_response.unwrap().val1, 7);

        // forward everything else the client sends
        while let Ok(m) = read_msg::<_, RemoteMessage>(&mut s).await {
            if seen_tx.send(m).is_err() {
                break;
            }
        }
    });

    let shared = Arc::new(Mutex::new(Shared::default()));
    let handle = tvremote::remote::spawn_on(
        &tokio::runtime::Handle::current(),
        "127.0.0.1".into(),
        port,
        client,
        shared.clone(),
        Arc::new(|| {}),
    );

    // wait for the session to become ready
    let mut ready = false;
    for _ in 0..100 {
        if shared.lock().unwrap().status == ConnStatus::Connected {
            ready = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(ready, "client never reached Connected: {:?}", shared.lock().unwrap().status);

    tokio::time::sleep(Duration::from_millis(100)).await;
    {
        let s = shared.lock().unwrap();
        assert_eq!(s.powered, Some(true));
        assert_eq!(s.volume, Some((37, 100, false)));
    }

    handle.key(23); // DPAD_CENTER
    handle.key_dir(24, DIR_START_LONG);
    handle.launch("https://www.youtube.com");

    let k = tokio::time::timeout(Duration::from_secs(3), seen_rx.recv()).await.unwrap().unwrap();
    assert_eq!(k.key_inject, Some(RemoteKeyInject { key_code: 23, direction: DIR_SHORT }));
    let k = tokio::time::timeout(Duration::from_secs(3), seen_rx.recv()).await.unwrap().unwrap();
    assert_eq!(k.key_inject, Some(RemoteKeyInject { key_code: 24, direction: DIR_START_LONG }));
    let k = tokio::time::timeout(Duration::from_secs(3), seen_rx.recv()).await.unwrap().unwrap();
    assert_eq!(k.app_link_launch.unwrap().app_link, "https://www.youtube.com");

    handle.disconnect();
}
