
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Read};
use std::path::Path;

pub struct HashVerifier;

impl HashVerifier {
    pub fn compute_file_hash<P: AsRef<Path>>(path: P) -> io::Result<String> {
        let mut file = fs::File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 4096];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    pub fn verify_file_integrity<P: AsRef<Path>>(path: P, expected_hash: &str) -> io::Result<bool> {
        let computed_hash = Self::compute_file_hash(path)?;
        Ok(computed_hash == expected_hash.to_lowercase())
    }

    pub fn compute_string_hash(data: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_string_hash_consistency() {
        let data = "test data for hashing";
        let hash1 = HashVerifier::compute_string_hash(data);
        let hash2 = HashVerifier::compute_string_hash(data);
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64);
    }

    #[test]
    fn test_file_hash_verification() -> io::Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let test_content = b"file integrity test content";
        temp_file.write_all(test_content)?;

        let computed_hash = HashVerifier::compute_file_hash(temp_file.path())?;
        let verification = HashVerifier::verify_file_integrity(temp_file.path(), &computed_hash)?;

        assert!(verification);
        Ok(())
    }

    #[test]
    fn test_hash_mismatch() -> io::Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(b"original content")?;

        let wrong_hash = "a" * 64;
        let verification = HashVerifier::verify_file_integrity(temp_file.path(), &wrong_hash)?;

        assert!(!verification);
        Ok(())
    }
}