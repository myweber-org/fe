use rand::Rng;
use std::error::Error;

pub struct KeyGenerator {
    rng: rand::rngs::ThreadRng,
}

impl KeyGenerator {
    pub fn new() -> Self {
        KeyGenerator {
            rng: rand::thread_rng(),
        }
    }

    pub fn generate_secure_key(&mut self, length: usize) -> Result<Vec<u8>, Box<dyn Error>> {
        if length < 16 {
            return Err("Key length must be at least 16 bytes".into());
        }

        let mut key = vec![0u8; length];
        self.rng.fill(&mut key[..]);

        Ok(key)
    }

    pub fn generate_hex_key(&mut self, length: usize) -> Result<String, Box<dyn Error>> {
        let key_bytes = self.generate_secure_key(length)?;
        Ok(hex::encode(key_bytes))
    }

    pub fn generate_base64_key(&mut self, length: usize) -> Result<String, Box<dyn Error>> {
        let key_bytes = self.generate_secure_key(length)?;
        Ok(base64::encode(key_bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let mut generator = KeyGenerator::new();
        
        let key = generator.generate_secure_key(32).unwrap();
        assert_eq!(key.len(), 32);

        let hex_key = generator.generate_hex_key(32).unwrap();
        assert_eq!(hex_key.len(), 64);

        let base64_key = generator.generate_base64_key(32).unwrap();
        assert!(!base64_key.is_empty());
    }

    #[test]
    fn test_invalid_key_length() {
        let mut generator = KeyGenerator::new();
        let result = generator.generate_secure_key(8);
        assert!(result.is_err());
    }
}