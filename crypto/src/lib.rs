#[macro_use]
extern crate lazy_static;

use std::fmt::{self, Display, Formatter};

pub use base64;
use ring::digest::{Context, SHA256};
use ring::hmac::{self, HMAC_SHA512, Key};
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use serde::de::Error;
use std::convert::TryFrom;

lazy_static! {
    static ref SECURE_RANDOM: SystemRandom = SystemRandom::new();
}

lazy_static! {
    static ref ARGON_CONFIG: argon2::Config<'static> = argon2::Config::default();
}

#[derive(Debug, Clone)]
pub struct SecureSecret(Vec<u8>);

impl SecureSecret {
    pub fn new<V: Into<Vec<u8>>>(bytes: V) -> SecureSecret {
        SecureSecret(bytes.into())
    }

    pub fn empty() -> SecureSecret {
        SecureSecret(vec![])
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl Display for SecureSecret {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}

impl TryFrom<String> for SecureSecret {
    type Error = hex::FromHexError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        hex::decode(value).map(SecureSecret)
    }
}

impl<'de> Deserialize<'de> for SecureSecret {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error> where
        D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        Self::try_from(s).map_err(<D as Deserializer<'de>>::Error::custom)
    }
}

impl Serialize for SecureSecret {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
        S: Serializer {
        self.to_string().serialize(serializer)
    }
}

pub fn generate_token(size: usize) -> Result<SecureSecret, String> {
    let mut buf: Vec<u8> = vec![0; size];
    SECURE_RANDOM.fill(&mut buf).map_err(|e| e.to_string())?;
    Ok(SecureSecret(buf))
}

pub fn generate_signature(source: &str, key: &str) -> SecureSecret {
    let key = Key::new(HMAC_SHA512, key.as_bytes());
    let sig = hmac::sign(&key, source.as_bytes());
    let buf = sig.as_ref().to_vec();
    SecureSecret(buf)
}

pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = generate_token(16)?;
    argon2::hash_encoded(password.as_bytes(), salt.as_bytes(), &ARGON_CONFIG)
        .map_err(|err| err.to_string())
}

pub fn verify_password(hash: &str, pwd: &str) -> Result<bool, String> {
    argon2::verify_encoded(hash, pwd.as_bytes()).map_err(|err| err.to_string())
}

pub fn sha256sum<S: AsRef<[u8]>>(s: S) -> SecureSecret {
    let mut ctx = Context::new(&SHA256);
    ctx.update(s.as_ref());
    SecureSecret::new(ctx.finish().as_ref())
}

pub fn base64url_encode<S: AsRef<[u8]>>(s: S) -> String {
    base64::encode_config(s, base64::URL_SAFE_NO_PAD)
}

pub fn pkce_challenge<V: AsRef<[u8]>>(code_verifier: V) -> String {
    let sha = sha256sum(code_verifier);
    let sha = sha.as_bytes();
    base64url_encode(sha)
}

#[cfg(test)]
mod test {
    use super::{
        generate_token, hash_password, pkce_challenge, sha256sum, verify_password,
    };
    use crate::SecureSecret;
    use std::convert::TryFrom;

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

    #[test]
    fn it_generates_a_sha256_checksum() {
        let s = "this is a test string";
        let exp_sha = "f6774519d1c7a3389ef327e9c04766b999db8cdfb85d1346c471ee86d65885bc";

        let sha = sha256sum(s);
        assert_eq!(exp_sha, sha.to_string());
    }

    #[test]
    fn it_encodes_a_pkce_verifier() {
        let verifier = "4a52ca3f5a6c4a47bb41c0c58105c3c2d848b69537464e8f86b9fb1f45815b9e2dadd0174fa440f89899dbab9d6f1400";
        let exp_challenge = "EaFiihM2I1egNwxqkmXd9WMww277yL-xFVhUZuU3kxY";
        let challenge = pkce_challenge(verifier);
        assert_eq!(exp_challenge, challenge);
    }

    #[test]
    fn it_decodes_from_hex() {
        let encoded = hex::encode("this is a test");
        let ss = SecureSecret::try_from(encoded);
        assert!(ss.is_ok());
        let secure_string = ss.unwrap().to_string();
        assert_eq!("this is a test", String::from_utf8(hex::decode(secure_string).unwrap()).unwrap());
    }
}
