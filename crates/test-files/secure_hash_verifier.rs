use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Error};
use std::path::Path;

pub fn calculate_file_hash(file_path: &str) -> Result<String, Error> {
    let path = Path::new(file_path);
    let mut file = File::open(path)?;
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

pub fn verify_file_integrity(file_path: &str, expected_hash: &str) -> Result<bool, Error> {
    let calculated_hash = calculate_file_hash(file_path)?;
    Ok(calculated_hash == expected_hash.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_hash_calculation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Test data for hash verification").unwrap();
        
        let hash_result = calculate_file_hash(temp_file.path().to_str().unwrap());
        assert!(hash_result.is_ok());
        
        let hash = hash_result.unwrap();
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_integrity_verification() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Consistent test data").unwrap();
        
        let hash = calculate_file_hash(temp_file.path().to_str().unwrap()).unwrap();
        let verification = verify_file_integrity(temp_file.path().to_str().unwrap(), &hash);
        
        assert!(verification.is_ok());
        assert!(verification.unwrap());
    }
}