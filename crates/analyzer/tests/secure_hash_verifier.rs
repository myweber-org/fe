use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Result};
use std::path::Path;

pub struct HashVerifier;

impl HashVerifier {
    pub fn compute_file_hash(file_path: &Path) -> Result<String> {
        let mut file = File::open(file_path)?;
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

    pub fn verify_file_integrity(file_path: &Path, expected_hash: &str) -> Result<bool> {
        let computed_hash = Self::compute_file_hash(file_path)?;
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
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_string_hash_consistency() {
        let data = "test_data";
        let hash1 = HashVerifier::compute_string_hash(data);
        let hash2 = HashVerifier::compute_string_hash(data);
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64);
    }

    #[test]
    fn test_file_hash_verification() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "sample content for hashing")?;

        let hash = HashVerifier::compute_file_hash(temp_file.path())?;
        let is_valid = HashVerifier::verify_file_integrity(temp_file.path(), &hash)?;

        assert!(is_valid);
        Ok(())
    }

    #[test]
    fn test_file_hash_mismatch() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "original content")?;

        let wrong_hash = "a" * 64;
        let is_valid = HashVerifier::verify_file_integrity(temp_file.path(), &wrong_hash)?;

        assert!(!is_valid);
        Ok(())
    }
}