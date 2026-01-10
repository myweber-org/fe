
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Error};
use std::path::Path;

pub fn calculate_file_hash(file_path: &Path) -> Result<String, Error> {
    let mut file = File::open(file_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];

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

pub fn verify_file_integrity(file_path: &Path, expected_hash: &str) -> Result<bool, Error> {
    let calculated_hash = calculate_file_hash(file_path)?;
    Ok(calculated_hash == expected_hash.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_hash_consistency() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Test data for hash verification").unwrap();
        
        let hash1 = calculate_file_hash(temp_file.path()).unwrap();
        let hash2 = calculate_file_hash(temp_file.path()).unwrap();
        
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_verification() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Verification test content").unwrap();
        
        let calculated_hash = calculate_file_hash(temp_file.path()).unwrap();
        let verification_result = verify_file_integrity(temp_file.path(), &calculated_hash).unwrap();
        
        assert!(verification_result);
    }

    #[test]
    fn test_hash_mismatch() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Original content").unwrap();
        
        let wrong_hash = "a" * 64; // Invalid SHA256 hash
        let verification_result = verify_file_integrity(temp_file.path(), &wrong_hash).unwrap();
        
        assert!(!verification_result);
    }
}