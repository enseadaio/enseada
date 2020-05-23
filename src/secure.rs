use std::fmt::{self, Display, Formatter};

use ring::hmac::{self, HMAC_SHA512, Key};
use ring::rand::{SecureRandom, SystemRandom};

use crate::config::CONFIG;

lazy_static! {
    static ref SECURE_RANDOM: SystemRandom = SystemRandom::new();
}

lazy_static! {
    static ref SECRET_KEY: Key = Key::new(HMAC_SHA512, CONFIG.secret_key().as_bytes());
}

lazy_static! {
    static ref ARGON_CONFIG: argon2::Config<'static> = argon2::Config::default();
}

#[derive(Debug, Clone)]
pub struct SecureSecret(Vec<u8>);

impl SecureSecret {
    pub fn new(bytes: Vec<u8>) -> SecureSecret {
        SecureSecret(bytes)
    }

    pub fn empty() -> SecureSecret {
        SecureSecret(vec![])
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_slice()
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

pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = generate_token(16)?;
    argon2::hash_encoded(password.as_bytes(), salt.as_bytes(), &ARGON_CONFIG)
        .map_err(|err| err.to_string())
}

pub fn verify_password(hash: &str, pwd: &str) -> Result<bool, String> {
    argon2::verify_encoded(hash, pwd.as_bytes())
        .map_err(|err| err.to_string())
}

#[cfg(test)]
mod test {
    use crate::secure::{generate_token, hash_password, verify_password};

    #[test]
    fn it_generates_a_token() {
        let r = generate_token(8);
        assert!(r.is_ok());
        let token = r.unwrap();
        assert_eq!(token.len(), 8);
    }

    #[test]
    fn it_hashes_a_password() {
        let pwd = "supersecretpassword";
        let r = hash_password(pwd);
        assert!(r.is_ok());
        let hash = r.unwrap();
        let r = verify_password(hash.as_str(), pwd);
        assert!(r.is_ok());
        assert!(r.unwrap());
    }
}