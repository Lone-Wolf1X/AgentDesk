//! Security primitives: constant-time token verification, self-signed TLS cert generation,
//! and certificate fingerprint calculation.

use crate::error::CoreError;
use rcgen::{CertificateParams, DistinguishedName, DnType, KeyPair};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

/// Holds a generated self-signed certificate and private key in PEM format.
#[derive(Clone, Debug)]
pub struct CertInfo {
    pub cert_pem: String,
    pub key_pem: String,
    pub sha256_fingerprint: String,
}

/// Compares two authentication tokens in constant time to prevent timing attacks.
pub fn constant_time_token_match(expected: &str, provided: &str) -> bool {
    let expected_bytes = expected.as_bytes();
    let provided_bytes = provided.as_bytes();

    if expected_bytes.len() != provided_bytes.len() {
        return false;
    }

    expected_bytes.ct_eq(provided_bytes).into()
}

/// Generates an in-memory self-signed X.509 certificate and private key for TLS transport.
pub fn generate_self_signed_cert(common_name: &str) -> Result<CertInfo, CoreError> {
    let key_pair = KeyPair::generate()
        .map_err(|e| CoreError::SecurityError(format!("Failed to generate key pair: {e}")))?;

    let mut params = CertificateParams::default();
    let mut dn = DistinguishedName::new();
    dn.push(DnType::CommonName, common_name);
    dn.push(DnType::OrganizationName, "AgentDesk Remote System");
    params.distinguished_name = dn;
    params.subject_alt_names = vec![
        rcgen::SanType::DnsName(common_name.to_string().try_into().map_err(|e| {
            CoreError::SecurityError(format!("Invalid DNS name {common_name}: {e}"))
        })?),
        rcgen::SanType::IpAddress(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)),
    ];

    let cert = params
        .self_signed(&key_pair)
        .map_err(|e| CoreError::SecurityError(format!("Failed to self-sign certificate: {e}")))?;

    let cert_pem = cert.pem();
    let key_pem = key_pair.serialize_pem();

    let cert_der = cert.der();
    let mut hasher = Sha256::new();
    hasher.update(cert_der.as_ref());
    let hash = hasher.finalize();

    let sha256_fingerprint = hash
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(":");

    Ok(CertInfo {
        cert_pem,
        key_pem,
        sha256_fingerprint,
    })
}

/// Calculates the SHA-256 fingerprint formatted with colon separators from raw DER bytes.
pub fn calculate_fingerprint(der: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(der);
    let hash = hasher.finalize();
    hash.iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(":")
}
