use ring::rand::{SecureRandom, SystemRandom};
use std::fmt::{self, Display, Formatter};
use ring::hmac::{self, Key, HMAC_SHA512};

use crate::config::CONFIG;

lazy_static! {
    static ref SECURE_RANDOM: SystemRandom = SystemRandom::new();
}

lazy_static! {
    static ref SECRET_KEY: Key = Key::new(HMAC_SHA512, CONFIG.secret_key().as_bytes());
}

#[derive(Debug, Clone)]
pub struct SecureSecret(Vec<u8>);

impl SecureSecret {
    pub fn new(bytes: Vec<u8>) -> SecureSecret {
        return SecureSecret(bytes)
    }
    pub fn empty() -> SecureSecret {
        SecureSecret(vec![])
    }
}

impl Display for SecureSecret {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}

pub fn generate_token(size: usize) -> Result<SecureSecret, String> {
    let mut buf: Vec<u8> = vec![0; size];
    SECURE_RANDOM.fill(&mut buf).map_err(|e| e.to_string())?;
    Ok(SecureSecret(buf))
}

pub fn generate_signature(source: &str) -> SecureSecret {
    let sig = hmac::sign(&SECRET_KEY, source.as_bytes());
    let buf = sig.as_ref().to_vec();
    SecureSecret(buf)
}

#[cfg(test)]
mod test {
    use crate::secure::generate_token;

    #[test]
    fn test() {
        let r = generate_token(8);
        assert!(r.is_ok());
        let token = r.unwrap();
        assert_eq!(token.len(), 8);
    }
}