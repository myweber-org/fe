use sha2::{Digest, Sha256};
use std::error::Error;

pub fn compute_sha256(data: &[u8]) -> Result<String, Box<dyn Error>> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

pub fn verify_sha256(data: &[u8], expected_hash: &str) -> Result<bool, Box<dyn Error>> {
    let computed_hash = compute_sha256(data)?;
    Ok(computed_hash == expected_hash.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_hashing() {
        let input = b"hello world";
        let hash = compute_sha256(input).unwrap();
        let expected = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        assert_eq!(hash, expected);
        assert!(verify_sha256(input, expected).unwrap());
    }

    #[test]
    fn test_sha256_verification_failure() {
        let input = b"hello world";
        let wrong_hash = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        assert!(!verify_sha256(input, wrong_hash).unwrap());
    }
}