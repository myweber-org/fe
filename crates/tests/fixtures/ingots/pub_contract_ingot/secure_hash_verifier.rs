use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Error};
use std::path::Path;

pub struct FileVerifier;

impl FileVerifier {
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

    pub fn verify_integrity<P: AsRef<Path>>(
        file_path: P,
        expected_hash: &str
    ) -> Result<bool, Error> {
        let calculated = Self::calculate_sha256(file_path)?;
        Ok(calculated == expected_hash.to_lowercase())
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
        writeln!(temp_file, "Test data for hashing").unwrap();
        
        let hash = FileVerifier::calculate_sha256(temp_file.path())
            .expect("Hash calculation failed");
        
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_integrity_verification() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Consistent data").unwrap();
        
        let hash = FileVerifier::calculate_sha256(temp_file.path())
            .unwrap();
        
        let valid = FileVerifier::verify_integrity(temp_file.path(), &hash)
            .expect("Verification failed");
        
        assert!(valid);
        
        let invalid = FileVerifier::verify_integrity(
            temp_file.path(),
            "0000000000000000000000000000000000000000000000000000000000000000"
        ).expect("Verification failed");
        
        assert!(!invalid);
    }
}