use std::io::Error as IoError;
use std::io::ErrorKind as IoErrorKind;
use std::path::PathBuf;

use tonic::transport::{Certificate, ClientTlsConfig, Identity};

#[derive(Default)]
pub struct TlsConfigBuilder {
    server_domain: Option<String>,
    ca: Option<String>,
    ca_path: Option<PathBuf>,
    client_cert: Option<String>,
    client_cert_path: Option<PathBuf>,
    client_key: Option<String>,
    client_key_path: Option<PathBuf>,
}

impl TlsConfigBuilder {
    pub fn server_domain<T: Into<String>>(&mut self, server_domain: T) -> &mut Self {
        self.server_domain = Some(server_domain.into());
        self
    }

    pub fn ca<T: Into<String>>(&mut self, ca: T) -> &mut Self {
        self.ca = Some(ca.into());
        self
    }

    pub fn ca_file<T: Into<PathBuf>>(&mut self, ca: T) -> &mut Self {
        self.ca_path = Some(ca.into());
        self
    }

    pub fn client_cert<T: Into<String>>(&mut self, client_cert: T) -> &mut Self {
        self.client_cert = Some(client_cert.into());
        self
    }

    pub fn client_cert_file<T: Into<PathBuf>>(&mut self, client_cert: T) -> &mut Self {
        self.client_cert_path = Some(client_cert.into());
        self
    }

    pub fn client_key<T: Into<String>>(&mut self, client_key: T) -> &mut Self {
        self.client_key = Some(client_key.into());
        self
    }

    pub fn client_key_file<T: Into<PathBuf>>(&mut self, client_key: T) -> &mut Self {
        self.client_key_path = Some(client_key.into());
        self
    }

    pub async fn build(self) -> Result<TlsConfig, IoError> {
        let server_domain = self.server_domain.ok_or_else(|| IoError::new(IoErrorKind::Other, "Server domain must be provided"))?;

        let ca = if let Some(path) = self.ca_path {
            tokio::fs::read(path).await?
        } else {
            self.ca.map(String::into_bytes).ok_or_else(|| IoError::new(IoErrorKind::Other, "CA certificate is required"))?
        };

        let client_cert = if let Some(path) = self.client_cert_path {
            Some(tokio::fs::read(path).await?)
        } else {
            self.client_cert.map(String::into_bytes)
        };

        let client_key = if let Some(path) = self.client_key_path {
            Some(tokio::fs::read(path).await?)
        } else {
            self.client_key.map(String::into_bytes)
        };

        let client_auth = client_cert.zip(client_key)
            .map(|(cert, key)| Identity::from_pem(cert, key));

        Ok(TlsConfig {
            server_domain,
            ca: Certificate::from_pem(ca),
            client_auth,
        })
    }
}

pub struct TlsConfig {
    ca: Certificate,
    server_domain: String,
    client_auth: Option<Identity>,
}

impl TlsConfig {
    pub fn new() -> TlsConfigBuilder {
        TlsConfigBuilder::default()
    }
}

impl Into<ClientTlsConfig> for TlsConfig {
    fn into(self) -> ClientTlsConfig {
        let cfg = ClientTlsConfig::new()
            .domain_name(self.server_domain)
            .ca_certificate(self.ca);

        if self.client_auth.is_none() {
            return cfg;
        }

        cfg.identity(self.client_auth.unwrap())
    }
}
