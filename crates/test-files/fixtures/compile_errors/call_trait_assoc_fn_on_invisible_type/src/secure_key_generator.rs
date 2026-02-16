use rand::Rng;
use std::error::Error;

pub fn generate_secure_key(length: usize) -> Result<Vec<u8>, Box<dyn Error>> {
    if length < 16 {
        return Err("Key length must be at least 16 bytes".into());
    }
    
    let mut rng = rand::thread_rng();
    let mut key = vec![0u8; length];
    rng.fill(&mut key[..]);
    
    Ok(key)
}

pub fn generate_hex_key(length: usize) -> Result<String, Box<dyn Error>> {
    let key = generate_secure_key(length)?;
    Ok(hex::encode(key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let key = generate_secure_key(32).unwrap();
        assert_eq!(key.len(), 32);
        
        let hex_key = generate_hex_key(32).unwrap();
        assert_eq!(hex_key.len(), 64);
    }

    #[test]
    fn test_invalid_length() {
        let result = generate_secure_key(8);
        assert!(result.is_err());
    }
}