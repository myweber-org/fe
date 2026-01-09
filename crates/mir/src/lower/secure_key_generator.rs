use rand::Rng;
use std::error::Error;

pub fn generate_secure_key(length: usize) -> Result<String, Box<dyn Error>> {
    if length < 16 {
        return Err("Key length must be at least 16 characters".into());
    }
    
    let mut rng = rand::thread_rng();
    let charset: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()-_=+[]{}|;:,.<>?"
        .chars()
        .collect();
    
    let key: String = (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..charset.len());
            charset[idx]
        })
        .collect();
    
    Ok(key)
}

pub fn generate_hex_key(bytes: usize) -> Result<String, Box<dyn Error>> {
    if bytes < 16 {
        return Err("Key must be at least 16 bytes".into());
    }
    
    let mut rng = rand::thread_rng();
    let mut buffer = vec![0u8; bytes];
    rng.fill(&mut buffer[..]);
    
    Ok(hex::encode(buffer))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_secure_key() {
        let key = generate_secure_key(32).unwrap();
        assert_eq!(key.len(), 32);
        
        let key2 = generate_secure_key(32).unwrap();
        assert_ne!(key, key2);
    }
    
    #[test]
    fn test_generate_hex_key() {
        let key = generate_hex_key(32).unwrap();
        assert_eq!(key.len(), 64);
        
        let key2 = generate_hex_key(32).unwrap();
        assert_ne!(key, key2);
    }
    
    #[test]
    fn test_invalid_length() {
        assert!(generate_secure_key(15).is_err());
        assert!(generate_hex_key(15).is_err());
    }
}