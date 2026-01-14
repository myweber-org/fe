use std::fs::File;
use std::io::{Read, Result};
use sha2::{Sha256, Digest};
use blake3::Hasher;

pub enum HashAlgorithm {
    Sha256,
    Blake3,
}

pub struct FileHasher;

impl FileHasher {
    pub fn calculate_hash(file_path: &str, algorithm: HashAlgorithm) -> Result<String> {
        let mut file = File::open(file_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        match algorithm {
            HashAlgorithm::Sha256 => {
                let mut hasher = Sha256::new();
                hasher.update(&buffer);
                Ok(format!("{:x}", hasher.finalize()))
            }
            HashAlgorithm::Blake3 => {
                let mut hasher = Hasher::new();
                hasher.update(&buffer);
                Ok(hasher.finalize().to_hex().to_string())
            }
        }
    }

    pub fn verify_hash(file_path: &str, expected_hash: &str, algorithm: HashAlgorithm) -> Result<bool> {
        let calculated = Self::calculate_hash(file_path, algorithm)?;
        Ok(calculated == expected_hash.to_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_sha256_verification() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "Test content for hashing")?;
        
        let hash = FileHasher::calculate_hash(temp_file.path().to_str().unwrap(), HashAlgorithm::Sha256)?;
        assert_eq!(hash.len(), 64);
        
        let valid = FileHasher::verify_hash(
            temp_file.path().to_str().unwrap(),
            &hash,
            HashAlgorithm::Sha256
        )?;
        assert!(valid);
        
        Ok(())
    }

    #[test]
    fn test_blake3_verification() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "Another test for Blake3")?;
        
        let hash = FileHasher::calculate_hash(temp_file.path().to_str().unwrap(), HashAlgorithm::Blake3)?;
        assert!(!hash.is_empty());
        
        let valid = FileHasher::verify_hash(
            temp_file.path().to_str().unwrap(),
            &hash,
            HashAlgorithm::Blake3
        )?;
        assert!(valid);
        
        Ok(())
    }
}