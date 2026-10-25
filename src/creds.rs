//! Client identity: a self-signed RSA certificate, generated once and stored.
//! The TV pins this certificate when pairing succeeds.

use anyhow::{anyhow, Result};
use rsa::pkcs8::EncodePrivateKey;
use rsa::RsaPrivateKey;
use std::path::Path;
use x509_parser::prelude::*;
use x509_parser::public_key::PublicKey;

#[derive(Clone)]
pub struct Creds {
    pub cert_der: Vec<u8>,
    pub key_der: Vec<u8>, // PKCS#8
}

impl Creds {
    pub fn load_or_create(dir: &Path) -> Result<Self> {
        std::fs::create_dir_all(dir)?;
        let cert_path = dir.join("client_cert.der");
        let key_path = dir.join("client_key.der");
        if let (Ok(cert_der), Ok(key_der)) = (std::fs::read(&cert_path), std::fs::read(&key_path)) {
            if !cert_der.is_empty() && !key_der.is_empty() {
                return Ok(Self { cert_der, key_der });
            }
        }
        let c = Self::generate()?;
        std::fs::write(&cert_path, &c.cert_der)?;
        std::fs::write(&key_path, &c.key_der)?;
        Ok(c)
    }

    pub fn generate() -> Result<Self> {
        let mut rng = rand::rngs::OsRng;
        let key = RsaPrivateKey::new(&mut rng, 2048)?;
        let pkcs8 = key.to_pkcs8_der().map_err(|e| anyhow!("pkcs8: {e}"))?;
        let key_der = pkcs8.as_bytes().to_vec();

        let pk = rustls::pki_types::PrivatePkcs8KeyDer::from(key_der.clone());
        let kp = rcgen::KeyPair::from_pkcs8_der_and_sign_algo(&pk, &rcgen::PKCS_RSA_SHA256)?;
        let mut params = rcgen::CertificateParams::new(Vec::<String>::new())?;
        params
            .distinguished_name
            .push(rcgen::DnType::CommonName, "tvremote");
        let cert = params.self_signed(&kp)?;
        Ok(Self { cert_der: cert.der().to_vec(), key_der })
    }

    /// (modulus, exponent) as minimal big-endian bytes.
    pub fn modulus_exponent(&self) -> Result<(Vec<u8>, Vec<u8>)> {
        rsa_modulus_exponent(&self.cert_der)
    }
}

fn strip_leading_zeros(b: &[u8]) -> Vec<u8> {
    let first = b.iter().position(|&x| x != 0).unwrap_or(b.len().saturating_sub(1));
    b[first..].to_vec()
}

/// Extract the RSA public key (modulus, exponent) from a DER certificate.
pub fn rsa_modulus_exponent(cert_der: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
    let (_, cert) = X509Certificate::from_der(cert_der).map_err(|e| anyhow!("x509: {e}"))?;
    match cert.public_key().parsed().map_err(|e| anyhow!("pubkey: {e}"))? {
        PublicKey::RSA(rsa) => Ok((strip_leading_zeros(rsa.modulus), strip_leading_zeros(rsa.exponent))),
        _ => Err(anyhow!("certificate does not contain an RSA key")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_and_extract() {
        let c = Creds::generate().unwrap();
        let (n, e) = c.modulus_exponent().unwrap();
        assert_eq!(n.len(), 256); // 2048-bit modulus
        assert_eq!(e, vec![1, 0, 1]); // 65537
        assert_ne!(n[0], 0);
    }
}
