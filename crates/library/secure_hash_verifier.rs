use std::fs::File;
use std::io::{Read, Result};
use sha2::{Sha256, Digest};
use blake3::Hasher as Blake3Hasher;

pub enum HashAlgorithm {
    SHA256,
    BLAKE3,
}

pub fn calculate_file_hash(path: &str, algorithm: HashAlgorithm) -> Result<String> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let hash = match algorithm {
        HashAlgorithm::SHA256 => {
            let mut hasher = Sha256::new();
            hasher.update(&buffer);
            format!("{:x}", hasher.finalize())
        }
        HashAlgorithm::BLAKE3 => {
            let mut hasher = Blake3Hasher::new();
            hasher.update(&buffer);
            format!("{}", hasher.finalize().to_hex())
        }
    };

    Ok(hash)
}

pub fn verify_file_integrity(
    path: &str, 
    expected_hash: &str, 
    algorithm: HashAlgorithm
) -> Result<bool> {
    let calculated_hash = calculate_file_hash(path, algorithm)?;
    Ok(calculated_hash == expected_hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_hash_calculation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Test data for hashing").unwrap();
        
        let sha256_hash = calculate_file_hash(
            temp_file.path().to_str().unwrap(), 
            HashAlgorithm::SHA256
        ).unwrap();
        
        let blake3_hash = calculate_file_hash(
            temp_file.path().to_str().unwrap(), 
            HashAlgorithm::BLAKE3
        ).unwrap();
        
        assert_eq!(sha256_hash.len(), 64);
        assert_eq!(blake3_hash.len(), 64);
    }

    #[test]
    fn test_integrity_verification() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Verification test").unwrap();
        
        let hash = calculate_file_hash(
            temp_file.path().to_str().unwrap(), 
            HashAlgorithm::SHA256
        ).unwrap();
        
        let is_valid = verify_file_integrity(
            temp_file.path().to_str().unwrap(), 
            &hash, 
            HashAlgorithm::SHA256
        ).unwrap();
        
        assert!(is_valid);
    }
}