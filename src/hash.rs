use std::fmt;

use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hash(pub [u8; 32]);

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&hex::encode(self.0))
    }
}

impl From<Vec<u8>> for Hash {
    fn from(value: Vec<u8>) -> Self {
        let mut digest = [0u8; 32];
        digest.copy_from_slice(&value);

        Self::from(digest)
    }
}

impl From<[u8; 32]> for Hash {
    fn from(value: [u8; 32]) -> Self {
        Self(value)
    }
}

impl From<String> for Hash {
    fn from(value: String) -> Self {
        Self::from(hex::decode(value).unwrap())
    }
}

impl From<&String> for Hash {
    fn from(value: &String) -> Self {
        Self::from(hex::decode(value).unwrap())
    }
}

impl From<&str> for Hash {
    fn from(value: &str) -> Self {
        Self::from(value.to_string())
    }
}

impl Hash {
    pub fn make<S, M>(salt: S, message: M) -> Self
    where
        S: AsRef<[u8]>,
        M: AsRef<[u8]>,
    {
        let mut hasher = Sha256::new();
        hasher.update(salt);
        hasher.update(message);

        Self::from(hasher.finalize().to_vec())
    }

    pub fn verify<S, M>(&self, salt: S, message: M) -> bool
    where
        S: AsRef<[u8]>,
        M: AsRef<[u8]>,
    {
        *self == Self::make(salt, message)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn verified() {
        let salt = Uuid::new_v4();
        let message = Uuid::new_v4();
        let hash = Hash::make(salt, message);

        assert!(hash.verify(salt, message));
    }

    #[test]
    fn unverified() {
        let salt = Uuid::new_v4();
        let message = "Hello World!";
        let hash = Hash::make(salt, message);

        assert!(!hash.verify(salt, "Hello World"));
    }
}
