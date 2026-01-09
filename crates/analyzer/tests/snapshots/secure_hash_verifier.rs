use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Result};
use std::path::Path;

pub struct HashVerifier;

impl HashVerifier {
    pub fn calculate_sha256<P: AsRef<Path>>(file_path: P) -> Result<String> {
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

    pub fn verify_file<P: AsRef<Path>>(file_path: P, expected_hash: &str) -> Result<bool> {
        let calculated = Self::calculate_sha256(file_path)?;
        Ok(calculated == expected_hash.to_lowercase())
    }

    pub fn compare_files<P: AsRef<Path>>(file1: P, file2: P) -> Result<bool> {
        let hash1 = Self::calculate_sha256(&file1)?;
        let hash2 = Self::calculate_sha256(&file2)?;
        Ok(hash1 == hash2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_identical_files() -> Result<()> {
        let mut file1 = NamedTempFile::new()?;
        let mut file2 = NamedTempFile::new()?;
        
        let content = b"Test content for hash verification";
        file1.write_all(content)?;
        file2.write_all(content)?;
        
        assert!(HashVerifier::compare_files(file1.path(), file2.path())?);
        Ok(())
    }

    #[test]
    fn test_different_files() -> Result<()> {
        let mut file1 = NamedTempFile::new()?;
        let mut file2 = NamedTempFile::new()?;
        
        file1.write_all(b"Content A")?;
        file2.write_all(b"Content B")?;
        
        assert!(!HashVerifier::compare_files(file1.path(), file2.path())?);
        Ok(())
    }

    #[test]
    fn test_hash_verification() -> Result<()> {
        let mut file = NamedTempFile::new()?;
        file.write_all(b"Hello, world!")?;
        
        let hash = HashVerifier::calculate_sha256(file.path())?;
        let expected = "315f5bdb76d078c43b8ac0064e4a0164612b1fce77c869345bfc94c75894edd3";
        
        assert!(HashVerifier::verify_file(file.path(), expected)?);
        Ok(())
    }
}