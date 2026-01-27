use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Error, ErrorKind};
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

    Ok(format!("{:x}", hasher.finalize()))
}

pub fn verify_file_integrity(original_path: &Path, comparison_path: &Path) -> Result<bool, Error> {
    let original_hash = calculate_file_hash(original_path)?;
    let comparison_hash = calculate_file_hash(comparison_path)?;

    if original_hash != comparison_hash {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!("Hash mismatch: {} != {}", original_hash, comparison_hash)
        ));
    }

    Ok(true)
}

pub fn verify_hash_string(file_path: &Path, expected_hash: &str) -> Result<bool, Error> {
    let actual_hash = calculate_file_hash(file_path)?;
    
    if actual_hash != expected_hash {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!("Hash mismatch: {} != {}", actual_hash, expected_hash)
        ));
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_identical_files() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        
        file1.write_all(b"test data").unwrap();
        file2.write_all(b"test data").unwrap();
        
        let result = verify_file_integrity(file1.path(), file2.path());
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_different_files() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        
        file1.write_all(b"test data 1").unwrap();
        file2.write_all(b"test data 2").unwrap();
        
        let result = verify_file_integrity(file1.path(), file2.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_hash_verification() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"test data").unwrap();
        
        let expected_hash = "916f0027a575074ce72a331777c3478d6513f786a591bd892da1a577bf2335f9";
        let result = verify_hash_string(file.path(), expected_hash);
        
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}