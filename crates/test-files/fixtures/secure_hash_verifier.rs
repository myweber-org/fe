
use sha2::{Sha256, Digest};
use hmac::{Hmac, Mac};
use std::fs::File;
use std::io::{Read, Result};
use std::path::Path;

type HmacSha256 = Hmac<Sha256>;

pub fn compute_file_hash(file_path: &Path) -> Result<String> {
    let mut file = File::open(file_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 4096];
    
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn verify_file_hash(file_path: &Path, expected_hash: &str) -> Result<bool> {
    let computed_hash = compute_file_hash(file_path)?;
    Ok(computed_hash == expected_hash.to_lowercase())
}

pub fn generate_hmac(key: &[u8], data: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(key)
        .expect("HMAC key should be valid");
    mac.update(data);
    format!("{:x}", mac.finalize().into_bytes())
}

pub fn verify_hmac(key: &[u8], data: &[u8], expected_mac: &str) -> bool {
    let computed_mac = generate_hmac(key, data);
    computed_mac == expected_mac.to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_file_hash_verification() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Test data for hashing").unwrap();
        
        let hash = compute_file_hash(temp_file.path()).unwrap();
        assert_eq!(hash.len(), 64);
        
        assert!(verify_file_hash(temp_file.path(), &hash).unwrap());
    }

    #[test]
    fn test_hmac_generation() {
        let key = b"secret-key";
        let data = b"important message";
        
        let hmac = generate_hmac(key, data);
        assert_eq!(hmac.len(), 64);
        
        assert!(verify_hmac(key, data, &hmac));
        assert!(!verify_hmac(b"wrong-key", data, &hmac));
    }
}