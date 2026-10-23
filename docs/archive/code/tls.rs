//! Optional TLS configuration via rustls.
//!
//! Enabled with the `tls` feature and `--tls-cert` / `--tls-key` CLI flags.

use std::path::Path;

/// TLS configuration for the HTTP engine.
pub struct TlsConfig {
    pub cert_path: String,
    pub key_path: String,
}

impl TlsConfig {
    /// Create a new TLS config, validating that cert and key files exist.
    pub fn new(cert_path: &str, key_path: &str) -> Result<Self, String> {
        if !Path::new(cert_path).exists() {
            return Err(format!("TLS certificate not found: {cert_path}"));
        }
        if !Path::new(key_path).exists() {
            return Err(format!("TLS key not found: {key_path}"));
        }
        Ok(Self {
            cert_path: cert_path.to_string(),
            key_path: key_path.to_string(),
        })
    }

    /// Build an axum-server RustlsConfig.
    pub async fn into_rustls_config(
        self,
    ) -> Result<axum_server::tls_rustls::RustlsConfig, std::io::Error> {
        axum_server::tls_rustls::RustlsConfig::from_pem_file(&self.cert_path, &self.key_path).await
    }
}
