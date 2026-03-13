use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub struct FileIntegrityChecker;

impl FileIntegrityChecker {
    pub fn calculate_sha256<P: AsRef<Path>>(path: P) -> Result<String, std::io::Error> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192];

        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    pub fn verify_file<P: AsRef<Path>>(path: P, expected_hash: &str) -> Result<bool, std::io::Error> {
        let calculated_hash = Self::calculate_sha256(path)?;
        Ok(calculated_hash == expected_hash.to_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_sha256_calculation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Test data for hash calculation").unwrap();
        
        let hash = FileIntegrityChecker::calculate_sha256(temp_file.path()).unwrap();
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_file_verification() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Verification test content").unwrap();
        
        let hash = FileIntegrityChecker::calculate_sha256(temp_file.path()).unwrap();
        let is_valid = FileIntegrityChecker::verify_file(temp_file.path(), &hash).unwrap();
        
        assert!(is_valid);
    }
}