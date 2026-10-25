//! TLS client with our own certificate; the TV's self-signed certificate is
//! accepted (trust comes from the pairing code, not from a CA).

use crate::creds::Creds;
use anyhow::{anyhow, Context, Result};
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::crypto::{ring, verify_tls12_signature, verify_tls13_signature, CryptoProvider};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName, UnixTime};
use rustls::{ClientConfig, DigitallySignedStruct, SignatureScheme};
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;
use tokio_rustls::TlsConnector;

#[derive(Debug)]
struct AcceptAnyServer(Arc<CryptoProvider>);

impl ServerCertVerifier for AcceptAnyServer {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        verify_tls12_signature(message, cert, dss, &self.0.signature_verification_algorithms)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        verify_tls13_signature(message, cert, dss, &self.0.signature_verification_algorithms)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.0.signature_verification_algorithms.supported_schemes()
    }
}

fn client_config(creds: &Creds) -> Result<ClientConfig> {
    let provider = Arc::new(ring::default_provider());
    let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(creds.key_der.clone()));
    let cfg = ClientConfig::builder_with_provider(provider.clone())
        .with_safe_default_protocol_versions()?
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(AcceptAnyServer(provider)))
        .with_client_auth_cert(vec![CertificateDer::from(creds.cert_der.clone())], key)?;
    Ok(cfg)
}

/// Open a TLS connection to `host:port` presenting our client certificate.
pub async fn connect(host: &str, port: u16, creds: &Creds) -> Result<TlsStream<TcpStream>> {
    let ip: IpAddr = host.trim().parse().map_err(|_| anyhow!("'{host}' is not a valid IP address"))?;
    let connector = TlsConnector::from(Arc::new(client_config(creds)?));
    let tcp = tokio::time::timeout(Duration::from_secs(6), TcpStream::connect((ip, port)))
        .await
        .map_err(|_| anyhow!("timed out connecting to {ip}:{port}"))?
        .with_context(|| format!("could not reach {ip}:{port}"))?;
    tcp.set_nodelay(true).ok();
    let tls = tokio::time::timeout(
        Duration::from_secs(8),
        connector.connect(ServerName::IpAddress(ip.into()), tcp),
    )
    .await
    .map_err(|_| anyhow!("TLS handshake timed out"))?
    .context("TLS handshake failed")?;
    Ok(tls)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_builds_with_generated_creds() {
        let c = Creds::generate().unwrap();
        client_config(&c).unwrap();
    }
}
