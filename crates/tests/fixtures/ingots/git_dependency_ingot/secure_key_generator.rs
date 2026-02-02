
use rand::Rng;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct KeyGenerator {
    length: usize,
    use_uppercase: bool,
    use_lowercase: bool,
    use_digits: bool,
    use_special: bool,
}

impl KeyGenerator {
    pub fn new(length: usize) -> Self {
        Self {
            length,
            use_uppercase: true,
            use_lowercase: true,
            use_digits: true,
            use_special: false,
        }
    }

    pub fn with_uppercase(mut self, enable: bool) -> Self {
        self.use_uppercase = enable;
        self
    }

    pub fn with_lowercase(mut self, enable: bool) -> Self {
        self.use_lowercase = enable;
        self
    }

    pub fn with_digits(mut self, enable: bool) -> Self {
        self.use_digits = enable;
        self
    }

    pub fn with_special(mut self, enable: bool) -> Self {
        self.use_special = enable;
        self
    }

    pub fn generate(&self) -> Result<String, Box<dyn Error>> {
        if self.length == 0 {
            return Err("Key length must be greater than 0".into());
        }

        let mut character_pool = Vec::new();
        
        if self.use_uppercase {
            character_pool.extend(b'A'..=b'Z');
        }
        if self.use_lowercase {
            character_pool.extend(b'a'..=b'z');
        }
        if self.use_digits {
            character_pool.extend(b'0'..=b'9');
        }
        if self.use_special {
            character_pool.extend(b"!@#$%^&*()-_=+[]{}|;:,.<>?");
        }

        if character_pool.is_empty() {
            return Err("At least one character set must be enabled".into());
        }

        let mut rng = rand::thread_rng();
        let mut result = String::with_capacity(self.length);

        for _ in 0..self.length {
            let idx = rng.gen_range(0..character_pool.len());
            result.push(character_pool[idx] as char);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_key_generation() {
        let generator = KeyGenerator::new(16);
        let key = generator.generate().unwrap();
        assert_eq!(key.len(), 16);
    }

    #[test]
    fn test_custom_character_sets() {
        let generator = KeyGenerator::new(10)
            .with_uppercase(true)
            .with_lowercase(false)
            .with_digits(true)
            .with_special(false);
        
        let key = generator.generate().unwrap();
        assert!(key.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()));
    }

    #[test]
    fn test_zero_length() {
        let generator = KeyGenerator::new(0);
        assert!(generator.generate().is_err());
    }

    #[test]
    fn test_no_character_sets() {
        let generator = KeyGenerator::new(10)
            .with_uppercase(false)
            .with_lowercase(false)
            .with_digits(false)
            .with_special(false);
        
        assert!(generator.generate().is_err());
    }
}