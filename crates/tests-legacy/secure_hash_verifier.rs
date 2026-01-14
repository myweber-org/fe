use std::fs::File;
use std::io::{Read, BufReader};
use sha2::{Sha256, Digest};

pub fn calculate_file_hash(filepath: &str) -> Result<String, std::io::Error> {
    let file = File::open(filepath)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0; 4096];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

pub fn verify_file_integrity(
    filepath: &str, 
    expected_hash: &str
) -> Result<bool, std::io::Error> {
    let calculated_hash = calculate_file_hash(filepath)?;
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
        writeln!(temp_file, "Test data for hashing").unwrap();
        
        let hash_result = calculate_file_hash(temp_file.path().to_str().unwrap());
        assert!(hash_result.is_ok());
        
        let hash = hash_result.unwrap();
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_integrity_verification() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Verification test data").unwrap();
        
        let calculated_hash = calculate_file_hash(
            temp_file.path().to_str().unwrap()
        ).unwrap();
        
        let verification_result = verify_file_integrity(
            temp_file.path().to_str().unwrap(),
            &calculated_hash
        ).unwrap();
        
        assert!(verification_result);
    }
}