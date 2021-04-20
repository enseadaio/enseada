use std::io::Error as IoError;
use std::io::ErrorKind as IoErrorKind;
use std::path::PathBuf;

#[derive(Default)]
pub struct TlsConfigBuilder {
    ca: Option<String>,
    ca_path: Option<PathBuf>,
    cert: Option<String>,
    cert_path: Option<PathBuf>,
    key: Option<String>,
    key_path: Option<PathBuf>,
}

impl TlsConfigBuilder {
    pub fn ca<T: Into<String>>(&mut self, ca: T) -> &mut Self {
        self.ca = Some(ca.into());
        self
    }

    pub fn ca_file<T: Into<PathBuf>>(&mut self, ca: T) -> &mut Self {
        self.ca_path = Some(ca.into());
        self
    }

    pub fn cert<T: Into<String>>(&mut self, cert: T) -> &mut Self {
        self.cert = Some(cert.into());
        self
    }

    pub fn cert_file<T: Into<PathBuf>>(&mut self, cert: T) -> &mut Self {
        self.cert_path = Some(cert.into());
        self
    }

    pub fn key<T: Into<String>>(&mut self, key: T) -> &mut Self {
        self.key = Some(key.into());
        self
    }

    pub fn key_file<T: Into<PathBuf>>(&mut self, key: T) -> &mut Self {
        self.key_path = Some(key.into());
        self
    }

    pub async fn build(self) -> Result<TlsConfig, IoError> {
        let ca = if let Some(path) = self.ca_path {
            tokio::fs::read(path).await?
        } else {
            self.ca.map(String::into_bytes).ok_or_else(|| IoError::new(IoErrorKind::Other, "CA certificate is required"))?
        };

        let cert = if let Some(path) = self.cert_path {
            tokio::fs::read(path).await?
        } else {
            self.cert.map(String::into_bytes).ok_or_else(|| IoError::new(IoErrorKind::Other, "TLS certificate is required"))?
        };

        let key = if let Some(path) = self.key_path {
            tokio::fs::read(path).await?
        } else {
            self.key.map(String::into_bytes).ok_or_else(|| IoError::new(IoErrorKind::Other, "TLS key is required"))?
        };

        Ok(TlsConfig {
            ca: Certificate::from_pem(ca),
            identity: Identity::from_pem(cert, key),
        })
    }
}

pub struct TlsConfig {
    ca: Certificate,
    identity: Identity,
}

impl TlsConfig {
    pub fn new() -> TlsConfigBuilder {
        TlsConfigBuilder::default()
    }
}

impl Into<ServerTlsConfig> for TlsConfig {
    fn into(self) -> ServerTlsConfig {
        ServerTlsConfig::new()
            .identity(self.identity)
            .client_ca_root(self.ca)
    }
}
