use std::fs::File;
use std::io::{Read, BufReader};
use std::path::Path;
use sha2::{Sha256, Digest};
use blake3::Hasher as Blake3Hasher;

pub enum HashAlgorithm {
    Sha256,
    Blake3,
}

pub struct FileHashVerifier;

impl FileHashVerifier {
    pub fn calculate_hash<P: AsRef<Path>>(
        path: P,
        algorithm: HashAlgorithm,
    ) -> Result<String, std::io::Error> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        
        match algorithm {
            HashAlgorithm::Sha256 => {
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
            HashAlgorithm::Blake3 => {
                let mut hasher = Blake3Hasher::new();
                let mut buffer = [0; 8192];
                
                loop {
                    let bytes_read = reader.read(&mut buffer)?;
                    if bytes_read == 0 {
                        break;
                    }
                    hasher.update(&buffer[..bytes_read]);
                }
                
                Ok(hasher.finalize().to_hex().to_string())
            }
        }
    }
    
    pub fn verify_hash<P: AsRef<Path>>(
        path: P,
        expected_hash: &str,
        algorithm: HashAlgorithm,
    ) -> Result<bool, std::io::Error> {
        let calculated = Self::calculate_hash(path, algorithm)?;
        Ok(calculated == expected_hash.to_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_sha256_verification() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Test content for hashing").unwrap();
        
        let hash = FileHashVerifier::calculate_hash(
            temp_file.path(),
            HashAlgorithm::Sha256
        ).unwrap();
        
        assert_eq!(hash.len(), 64);
        
        let is_valid = FileHashVerifier::verify_hash(
            temp_file.path(),
            &hash,
            HashAlgorithm::Sha256
        ).unwrap();
        
        assert!(is_valid);
    }
    
    #[test]
    fn test_blake3_verification() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Another test for Blake3").unwrap();
        
        let hash = FileHashVerifier::calculate_hash(
            temp_file.path(),
            HashAlgorithm::Blake3
        ).unwrap();
        
        assert!(hash.len() >= 64);
        
        let is_valid = FileHashVerifier::verify_hash(
            temp_file.path(),
            &hash,
            HashAlgorithm::Blake3
        ).unwrap();
        
        assert!(is_valid);
    }
}use std::fs::File;
use std::io::{Read, Result};
use sha2::{Sha256, Digest};

pub fn compute_file_hash(file_path: &str) -> Result<String> {
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

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

pub fn verify_file_hash(file_path: &str, expected_hash: &str) -> Result<bool> {
    let computed_hash = compute_file_hash(file_path)?;
    Ok(computed_hash == expected_hash.to_lowercase())
}