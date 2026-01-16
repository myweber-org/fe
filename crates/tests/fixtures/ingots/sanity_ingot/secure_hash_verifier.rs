
use sha2::{Sha256, Digest};
use hmac::{Hmac, Mac};
use std::fs::File;
use std::io::{Read, BufReader};
use std::path::Path;

type HmacSha256 = Hmac<Sha256>;

pub fn compute_file_hash(file_path: &Path) -> Result<String, std::io::Error> {
    let file = File::open(file_path)?;
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
    
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn verify_file_hash(file_path: &Path, expected_hash: &str) -> Result<bool, std::io::Error> {
    let computed_hash = compute_file_hash(file_path)?;
    Ok(computed_hash == expected_hash.to_lowercase())
}

pub fn generate_hmac(key: &[u8], data: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(key)
        .expect("HMAC key length must be valid");
    mac.update(data);
    format!("{:x}", mac.finalize().into_bytes())
}

pub fn verify_hmac(key: &[u8], data: &[u8], expected_hmac: &str) -> bool {
    let computed_hmac = generate_hmac(key, data);
    computed_hmac == expected_hmac.to_lowercase()
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
        
        let hash = compute_file_hash(temp_file.path()).unwrap();
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_hash_verification() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Verification test").unwrap();
        
        let hash = compute_file_hash(temp_file.path()).unwrap();
        let is_valid = verify_file_hash(temp_file.path(), &hash).unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_hmac_generation() {
        let key = b"secret-key";
        let data = b"message to authenticate";
        let hmac = generate_hmac(key, data);
        assert_eq!(hmac.len(), 64);
    }

    #[test]
    fn test_hmac_verification() {
        let key = b"secret-key";
        let data = b"message to authenticate";
        let hmac = generate_hmac(key, data);
        assert!(verify_hmac(key, data, &hmac));
    }
}