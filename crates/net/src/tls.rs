//! TLS certificate management, configuration, and fingerprint verification with Rustls.

use crate::error::NetError;
use agentdesk_core::security::{generate_self_signed_cert, CertInfo};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName};
use rustls::server::ServerConfig;
use rustls::{ClientConfig, Error as RustlsError, SignatureScheme};
use std::sync::Arc;
use tracing::info;

/// Creates server TLS configuration using an auto-generated self-signed certificate.
pub fn create_server_tls_config() -> Result<(Arc<ServerConfig>, CertInfo), NetError> {
    let _ = rustls::crypto::ring::default_provider().install_default();

    let cert_info = generate_self_signed_cert("agentdesk.local")
        .map_err(|e| NetError::TlsError(e.to_string()))?;

    let cert_der = rustls_pemfile::certs(&mut cert_info.cert_pem.as_bytes())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| NetError::TlsError(format!("Failed to parse certificate PEM: {e}")))?;

    let key_der = rustls_pemfile::private_key(&mut cert_info.key_pem.as_bytes())
        .map_err(|e| NetError::TlsError(format!("Failed to parse private key PEM: {e}")))?
        .ok_or_else(|| NetError::TlsError("No private key found in PEM".to_string()))?;

    let server_config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_der, key_der)
        .map_err(|e| NetError::TlsError(format!("Failed to build ServerConfig: {e}")))?;

    info!("Generated TLS self-signed cert. SHA-256 Fingerprint: {}", cert_info.sha256_fingerprint);

    Ok((Arc::new(server_config), cert_info))
}

/// A custom ServerCertVerifier that checks the SHA-256 fingerprint or accepts self-signed certs.
#[derive(Debug)]
struct FingerprintCertVerifier {
    expected_fingerprint: Option<String>,
}

impl rustls::client::danger::ServerCertVerifier for FingerprintCertVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, RustlsError> {
        let actual_fp = agentdesk_core::security::calculate_fingerprint(end_entity.as_ref());

        if let Some(ref expected) = self.expected_fingerprint {
            if actual_fp.eq_ignore_ascii_case(expected) {
                Ok(rustls::client::danger::ServerCertVerified::assertion())
            } else {
                Err(RustlsError::General(format!(
                    "Certificate fingerprint mismatch: expected {expected}, got {actual_fp}"
                )))
            }
        } else {
            // Self-signed allowed in Phase 1 if no fingerprint pinned
            Ok(rustls::client::danger::ServerCertVerified::assertion())
        }
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, RustlsError> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, RustlsError> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![
            SignatureScheme::ECDSA_NISTP256_SHA256,
            SignatureScheme::ECDSA_NISTP384_SHA384,
            SignatureScheme::ED25519,
            SignatureScheme::RSA_PSS_SHA256,
            SignatureScheme::RSA_PKCS1_SHA256,
        ]
    }
}

/// Creates client TLS configuration, optionally pinning expected SHA-256 fingerprint.
pub fn create_client_tls_config(expected_fingerprint: Option<String>) -> Arc<ClientConfig> {
    let _ = rustls::crypto::ring::default_provider().install_default();

    let verifier = Arc::new(FingerprintCertVerifier { expected_fingerprint });

    let config = ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(verifier)
        .with_no_client_auth();

    Arc::new(config)
}
