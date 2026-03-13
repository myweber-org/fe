
use sha2::{Digest, Sha256};
use std::error::Error;

pub fn generate_secure_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    format!("{:x}", result)
}

pub fn verify_hash(data: &[u8], expected_hash: &str) -> bool {
    generate_secure_hash(data) == expected_hash
}

pub fn hash_file_contents(path: &str) -> Result<String, Box<dyn Error>> {
    let contents = std::fs::read(path)?;
    Ok(generate_secure_hash(&contents))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_generation() {
        let data = b"test data";
        let hash = generate_secure_hash(data);
        assert_eq!(hash.len(), 64);
        assert!(verify_hash(data, &hash));
    }

    #[test]
    fn test_hash_verification() {
        let data = b"verification test";
        let hash = generate_secure_hash(data);
        assert!(verify_hash(data, &hash));
        assert!(!verify_hash(b"wrong data", &hash));
    }
}