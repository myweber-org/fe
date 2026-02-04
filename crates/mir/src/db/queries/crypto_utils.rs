
use sha2::{Digest, Sha256};
use rand::RngCore;

/// Generates a cryptographically secure random byte array of the specified length.
pub fn generate_random_bytes(length: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let mut bytes = vec![0u8; length];
    rng.fill_bytes(&mut bytes);
    bytes
}

/// Computes the SHA-256 hash of the input data and returns it as a hex string.
pub fn sha256_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_random_bytes_length() {
        let bytes = generate_random_bytes(32);
        assert_eq!(bytes.len(), 32);
    }

    #[test]
    fn test_sha256_hash() {
        let data = b"hello world";
        let hash = sha256_hash(data);
        let expected = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        assert_eq!(hash, expected);
    }
}
use rand::Rng;
use sha2::{Sha256, Digest};

pub fn generate_salt() -> [u8; 16] {
    let mut rng = rand::thread_rng();
    let mut salt = [0u8; 16];
    rng.fill(&mut salt);
    salt
}

pub fn hash_password(password: &str, salt: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt);
    let result = hasher.finalize();
    hex::encode(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_salt() {
        let salt = generate_salt();
        assert_eq!(salt.len(), 16);
    }

    #[test]
    fn test_hash_password() {
        let password = "secure_password123";
        let salt = generate_salt();
        let hash = hash_password(password, &salt);
        assert_eq!(hash.len(), 64);
    }
}