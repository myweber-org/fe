
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Error, ErrorKind};
use std::path::Path;

pub struct HashVerifier;

impl HashVerifier {
    pub fn compute_file_hash(file_path: &str) -> Result<String, Error> {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(Error::new(ErrorKind::NotFound, "File not found"));
        }

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

    pub fn verify_file_hash(file_path: &str, expected_hash: &str) -> Result<bool, Error> {
        let computed_hash = Self::compute_file_hash(file_path)?;
        Ok(computed_hash == expected_hash.to_lowercase())
    }

    pub fn compare_files(file1: &str, file2: &str) -> Result<bool, Error> {
        let hash1 = Self::compute_file_hash(file1)?;
        let hash2 = Self::compute_file_hash(file2)?;
        Ok(hash1 == hash2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_hash_computation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Test data for hashing").unwrap();
        
        let hash = HashVerifier::compute_file_hash(temp_file.path().to_str().unwrap())
            .expect("Should compute hash");
        
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_hash_verification() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Consistent data").unwrap();
        
        let known_hash = "a1b2c3d4e5f678901234567890abcdef1234567890abcdef1234567890abcdef";
        let result = HashVerifier::verify_file_hash(
            temp_file.path().to_str().unwrap(),
            known_hash
        );
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_file_comparison() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        
        writeln!(file1, "Same content").unwrap();
        writeln!(file2, "Same content").unwrap();
        
        let result = HashVerifier::compare_files(
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap()
        ).unwrap();
        
        assert!(result);
    }
}