use sha2::{Digest, Sha256};
use rand::{RngCore, rngs::OsRng};
use std::error::Error;

pub struct SecureHasher {
    salt: [u8; 32],
}

impl SecureHasher {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let mut salt = [0u8; 32];
        OsRng.fill_bytes(&mut salt);
        Ok(Self { salt })
    }

    pub fn with_salt(salt: [u8; 32]) -> Self {
        Self { salt }
    }

    pub fn generate_hash(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(&self.salt);
        hasher.update(data);
        let result = hasher.finalize();
        hex::encode(result)
    }

    pub fn verify(&self, data: &[u8], expected_hash: &str) -> bool {
        self.generate_hash(data) == expected_hash
    }

    pub fn get_salt(&self) -> &[u8] {
        &self.salt
    }
}

pub fn hash_password(password: &str) -> Result<(String, [u8; 32]), Box<dyn Error>> {
    let hasher = SecureHasher::new()?;
    let hash = hasher.generate_hash(password.as_bytes());
    Ok((hash, hasher.salt))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_generation() {
        let hasher = SecureHasher::new().unwrap();
        let data = b"test_data";
        let hash = hasher.generate_hash(data);
        assert_eq!(hash.len(), 64);
        assert!(hasher.verify(data, &hash));
    }

    #[test]
    fn test_different_salts_produce_different_hashes() {
        let hasher1 = SecureHasher::new().unwrap();
        let hasher2 = SecureHasher::new().unwrap();
        let data = b"same_data";
        
        assert_ne!(
            hasher1.generate_hash(data),
            hasher2.generate_hash(data)
        );
    }

    #[test]
    fn test_password_hashing() {
        let (hash, salt) = hash_password("secure_password").unwrap();
        let hasher = SecureHasher::with_salt(salt);
        assert!(hasher.verify("secure_password".as_bytes(), &hash));
        assert!(!hasher.verify("wrong_password".as_bytes(), &hash));
    }
}