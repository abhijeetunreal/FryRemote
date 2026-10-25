//! Pairing flow (TV port 6467). The TV shows a 6-character hex code; the
//! secret we send proves we saw it, bound to both certificates' RSA keys.

use crate::creds::{rsa_modulus_exponent, Creds};
use crate::proto::*;
use crate::tls;
use crate::wire::{read_msg, write_msg};
use anyhow::{anyhow, bail, Result};
use sha2::{Digest, Sha256};
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;

pub const PAIRING_PORT: u16 = 6467;

pub struct PairingSession {
    stream: TlsStream<TcpStream>,
    client_key: (Vec<u8>, Vec<u8>),
    server_key: (Vec<u8>, Vec<u8>),
}

#[derive(Debug)]
pub enum CodeError {
    /// The code failed the local check byte test — typo, safe to retry.
    WrongCode,
    Other(anyhow::Error),
}

fn expect_ok(m: &PairingMessage, what: &str) -> Result<()> {
    if m.status != STATUS_OK {
        bail!("TV rejected {what} (status {})", m.status);
    }
    Ok(())
}

fn envelope() -> PairingMessage {
    PairingMessage {
        protocol_version: PAIRING_PROTOCOL_VERSION,
        status: STATUS_OK,
        ..Default::default()
    }
}

/// secret = SHA256(client_n || client_e || server_n || server_e || code[1..])
pub fn compute_secret(
    client: &(Vec<u8>, Vec<u8>),
    server: &(Vec<u8>, Vec<u8>),
    code_bytes: &[u8],
) -> Vec<u8> {
    let mut h = Sha256::new();
    h.update(&client.0);
    h.update(&client.1);
    h.update(&server.0);
    h.update(&server.1);
    h.update(&code_bytes[1..]);
    h.finalize().to_vec()
}

pub fn parse_code(code: &str) -> Option<Vec<u8>> {
    let c: String = code.trim().chars().filter(|c| !c.is_whitespace()).collect();
    if c.len() != 6 {
        return None;
    }
    hex::decode(c).ok()
}

impl PairingSession {
    /// Connect and run the handshake up to the point where the TV shows its code.
    pub async fn start(host: &str, creds: &Creds, client_name: &str) -> Result<Self> {
        Self::start_on(host, PAIRING_PORT, creds, client_name).await
    }

    pub async fn start_on(host: &str, port: u16, creds: &Creds, client_name: &str) -> Result<Self> {
        let mut s = tls::connect(host, port, creds).await?;

        let server_cert = s
            .get_ref()
            .1
            .peer_certificates()
            .and_then(|c| c.first())
            .ok_or_else(|| anyhow!("TV sent no certificate"))?
            .to_vec();
        let server_key = rsa_modulus_exponent(&server_cert)?;
        let client_key = creds.modulus_exponent()?;

        // 1. pairing request
        let mut m = envelope();
        m.pairing_request = Some(PairingRequest {
            service_name: "androidtvremote2".into(),
            client_name: client_name.into(),
        });
        write_msg(&mut s, &m).await?;
        let ack: PairingMessage = read_msg(&mut s).await?;
        expect_ok(&ack, "pairing request")?;

        // 2. options
        let enc = PairingEncoding { r#type: ENCODING_HEXADECIMAL, symbol_length: 6 };
        let mut m = envelope();
        m.options = Some(PairingOptions {
            input_encodings: vec![enc.clone()],
            output_encodings: vec![],
            preferred_role: ROLE_INPUT,
        });
        write_msg(&mut s, &m).await?;
        let opts: PairingMessage = read_msg(&mut s).await?;
        expect_ok(&opts, "options")?;

        // 3. configuration -> TV displays the code
        let mut m = envelope();
        m.configuration = Some(PairingConfiguration { encoding: Some(enc), client_role: ROLE_INPUT });
        write_msg(&mut s, &m).await?;
        let cfg: PairingMessage = read_msg(&mut s).await?;
        expect_ok(&cfg, "configuration")?;

        Ok(Self { stream: s, client_key, server_key })
    }

    /// Submit the code shown on the TV.
    pub async fn finish(&mut self, code: &str) -> std::result::Result<(), CodeError> {
        let bytes = parse_code(code).ok_or(CodeError::WrongCode)?;
        let secret = compute_secret(&self.client_key, &self.server_key, &bytes);
        // The first byte of the hash is the code's check byte.
        if secret[0] != bytes[0] {
            return Err(CodeError::WrongCode);
        }
        let mut m = envelope();
        m.secret = Some(PairingSecret { secret });
        write_msg(&mut self.stream, &m).await.map_err(CodeError::Other)?;
        let ack: PairingMessage = read_msg(&mut self.stream).await.map_err(CodeError::Other)?;
        expect_ok(&ack, "secret").map_err(CodeError::Other)?;
        if ack.secret_ack.is_none() {
            return Err(CodeError::Other(anyhow!("TV did not acknowledge the secret")));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_parsing() {
        assert_eq!(parse_code("1A2b3C"), Some(vec![0x1a, 0x2b, 0x3c]));
        assert_eq!(parse_code(" 1a 2b 3c "), Some(vec![0x1a, 0x2b, 0x3c]));
        assert_eq!(parse_code("12345"), None);
        assert_eq!(parse_code("zzzzzz"), None);
    }

    #[test]
    fn secret_is_bound_to_inputs() {
        let c = (vec![1, 2, 3], vec![1, 0, 1]);
        let s = (vec![9, 8, 7], vec![1, 0, 1]);
        let a = compute_secret(&c, &s, &[0xaa, 1, 2]);
        let b = compute_secret(&c, &s, &[0xaa, 1, 3]);
        let d = compute_secret(&s, &c, &[0xaa, 1, 2]);
        assert_eq!(a.len(), 32);
        assert_ne!(a, b);
        assert_ne!(a, d);
        // check byte (code[0]) is not part of the hash input
        assert_eq!(a, compute_secret(&c, &s, &[0x00, 1, 2]));
    }
}
