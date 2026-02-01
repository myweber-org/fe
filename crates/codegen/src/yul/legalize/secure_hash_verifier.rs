
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Error};
use std::path::Path;

pub struct FileIntegrityChecker;

impl FileIntegrityChecker {
    pub fn calculate_sha256<P: AsRef<Path>>(path: P) -> Result<String, Error> {
        let mut file = File::open(path)?;
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

    pub fn verify_file<P: AsRef<Path>>(path: P, expected_hash: &str) -> Result<bool, Error> {
        let calculated_hash = Self::calculate_sha256(path)?;
        Ok(calculated_hash == expected_hash.to_lowercase())
    }

    pub fn compare_files<P: AsRef<Path>>(path1: P, path2: P) -> Result<bool, Error> {
        let hash1 = Self::calculate_sha256(path1)?;
        let hash2 = Self::calculate_sha256(path2)?;
        Ok(hash1 == hash2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_identical_files() -> Result<(), Error> {
        let mut file1 = NamedTempFile::new()?;
        let mut file2 = NamedTempFile::new()?;
        
        let test_data = b"Test data for hash verification";
        file1.write_all(test_data)?;
        file2.write_all(test_data)?;
        
        assert!(FileIntegrityChecker::compare_files(file1.path(), file2.path())?);
        Ok(())
    }

    #[test]
    fn test_different_files() -> Result<(), Error> {
        let mut file1 = NamedTempFile::new()?;
        let mut file2 = NamedTempFile::new()?;
        
        file1.write_all(b"Data 1")?;
        file2.write_all(b"Data 2")?;
        
        assert!(!FileIntegrityChecker::compare_files(file1.path(), file2.path())?);
        Ok(())
    }

    #[test]
    fn test_hash_verification() -> Result<(), Error> {
        let mut file = NamedTempFile::new()?;
        file.write_all(b"Hello, world!")?;
        
        let hash = FileIntegrityChecker::calculate_sha256(file.path())?;
        let expected_hash = "315f5bdb76d078c43b8ac0064e4a0164612b1fce77c869345bfc94c75894edd3";
        
        assert!(FileIntegrityChecker::verify_file(file.path(), expected_hash)?);
        Ok(())
    }
}