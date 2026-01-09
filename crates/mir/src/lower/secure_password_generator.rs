use rand::{thread_rng, Rng};
use std::collections::HashSet;

pub struct PasswordGenerator {
    length: usize,
    use_uppercase: bool,
    use_lowercase: bool,
    use_digits: bool,
    use_special: bool,
}

impl PasswordGenerator {
    pub fn new(length: usize) -> Self {
        Self {
            length,
            use_uppercase: true,
            use_lowercase: true,
            use_digits: true,
            use_special: true,
        }
    }

    pub fn uppercase(mut self, enable: bool) -> Self {
        self.use_uppercase = enable;
        self
    }

    pub fn lowercase(mut self, enable: bool) -> Self {
        self.use_lowercase = enable;
        self
    }

    pub fn digits(mut self, enable: bool) -> Self {
        self.use_digits = enable;
        self
    }

    pub fn special(mut self, enable: bool) -> Self {
        self.use_special = enable;
        self
    }

    pub fn generate(&self) -> Result<String, &'static str> {
        if self.length == 0 {
            return Err("Password length must be greater than 0");
        }

        if !self.use_uppercase && !self.use_lowercase && !self.use_digits && !self.use_special {
            return Err("At least one character set must be enabled");
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

        let mut rng = thread_rng();
        let mut password_chars = Vec::with_capacity(self.length);
        let mut used_chars = HashSet::new();

        for _ in 0..self.length {
            let idx = rng.gen_range(0..character_pool.len());
            let ch = character_pool[idx] as char;
            password_chars.push(ch);
            used_chars.insert(ch);
        }

        let password: String = password_chars.into_iter().collect();
        
        Ok(password)
    }

    pub fn validate_strength(&self, password: &str) -> bool {
        if password.len() < self.length {
            return false;
        }

        let mut has_upper = false;
        let mut has_lower = false;
        let mut has_digit = false;
        let mut has_special = false;

        for ch in password.chars() {
            if ch.is_ascii_uppercase() {
                has_upper = true;
            } else if ch.is_ascii_lowercase() {
                has_lower = true;
            } else if ch.is_ascii_digit() {
                has_digit = true;
            } else if ch.is_ascii_punctuation() {
                has_special = true;
            }
        }

        (!self.use_uppercase || has_upper) &&
        (!self.use_lowercase || has_lower) &&
        (!self.use_digits || has_digit) &&
        (!self.use_special || has_special)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_generation() {
        let generator = PasswordGenerator::new(12);
        let password = generator.generate().unwrap();
        assert_eq!(password.len(), 12);
        assert!(generator.validate_strength(&password));
    }

    #[test]
    fn test_custom_character_sets() {
        let generator = PasswordGenerator::new(8)
            .uppercase(false)
            .special(false);
        
        let password = generator.generate().unwrap();
        assert!(password.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()));
    }

    #[test]
    fn test_invalid_configuration() {
        let generator = PasswordGenerator::new(0);
        assert!(generator.generate().is_err());

        let generator = PasswordGenerator::new(10)
            .uppercase(false)
            .lowercase(false)
            .digits(false)
            .special(false);
        
        assert!(generator.generate().is_err());
    }
}use rand::Rng;
use rand::rngs::OsRng;

const DEFAULT_LENGTH: usize = 16;
const UPPERCASE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWERCASE: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
const NUMBERS: &[u8] = b"0123456789";
const SYMBOLS: &[u8] = b"!@#$%^&*()_+-=[]{}|;:,.<>?";

pub struct PasswordGenerator {
    length: usize,
    use_uppercase: bool,
    use_lowercase: bool,
    use_numbers: bool,
    use_symbols: bool,
}

impl PasswordGenerator {
    pub fn new() -> Self {
        Self {
            length: DEFAULT_LENGTH,
            use_uppercase: true,
            use_lowercase: true,
            use_numbers: true,
            use_symbols: true,
        }
    }

    pub fn length(mut self, length: usize) -> Self {
        self.length = length;
        self
    }

    pub fn uppercase(mut self, enable: bool) -> Self {
        self.use_uppercase = enable;
        self
    }

    pub fn lowercase(mut self, enable: bool) -> Self {
        self.use_lowercase = enable;
        self
    }

    pub fn numbers(mut self, enable: bool) -> Self {
        self.use_numbers = enable;
        self
    }

    pub fn symbols(mut self, enable: bool) -> Self {
        self.use_symbols = enable;
        self
    }

    pub fn generate(&self) -> Result<String, &'static str> {
        if self.length == 0 {
            return Err("Password length must be greater than zero");
        }

        let mut character_pool = Vec::new();
        if self.use_uppercase {
            character_pool.extend_from_slice(UPPERCASE);
        }
        if self.use_lowercase {
            character_pool.extend_from_slice(LOWERCASE);
        }
        if self.use_numbers {
            character_pool.extend_from_slice(NUMBERS);
        }
        if self.use_symbols {
            character_pool.extend_from_slice(SYMBOLS);
        }

        if character_pool.is_empty() {
            return Err("At least one character set must be enabled");
        }

        let mut rng = OsRng;
        let password: String = (0..self.length)
            .map(|_| {
                let idx = rng.gen_range(0..character_pool.len());
                character_pool[idx] as char
            })
            .collect();

        Ok(password)
    }
}

impl Default for PasswordGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_generator() {
        let generator = PasswordGenerator::new();
        let password = generator.generate().unwrap();
        assert_eq!(password.len(), DEFAULT_LENGTH);
    }

    #[test]
    fn test_custom_length() {
        let generator = PasswordGenerator::new().length(20);
        let password = generator.generate().unwrap();
        assert_eq!(password.len(), 20);
    }

    #[test]
    fn test_only_numbers() {
        let generator = PasswordGenerator::new()
            .uppercase(false)
            .lowercase(false)
            .symbols(false)
            .numbers(true);
        let password = generator.generate().unwrap();
        assert!(password.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_no_character_sets() {
        let generator = PasswordGenerator::new()
            .uppercase(false)
            .lowercase(false)
            .symbols(false)
            .numbers(false);
        let result = generator.generate();
        assert!(result.is_err());
    }

    #[test]
    fn test_zero_length() {
        let generator = PasswordGenerator::new().length(0);
        let result = generator.generate();
        assert!(result.is_err());
    }
}