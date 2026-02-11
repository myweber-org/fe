
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Error};
use std::path::Path;

pub struct HashVerifier;

impl HashVerifier {
    pub fn calculate_file_hash(file_path: &Path) -> Result<String, Error> {
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

    pub fn verify_file_integrity(file_path: &Path, expected_hash: &str) -> Result<bool, Error> {
        let calculated_hash = Self::calculate_file_hash(file_path)?;
        Ok(calculated_hash == expected_hash.to_lowercase())
    }

    pub fn generate_hash(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_hash_generation() {
        let data = b"test data";
        let hash = HashVerifier::generate_hash(data);
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_file_verification() -> Result<(), Error> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "Test content for verification")?;
        
        let hash = HashVerifier::calculate_file_hash(temp_file.path())?;
        let is_valid = HashVerifier::verify_file_integrity(temp_file.path(), &hash)?;
        
        assert!(is_valid);
        Ok(())
    }
}