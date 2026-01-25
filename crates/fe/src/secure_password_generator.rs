use rand::Rng;

pub fn generate_password(length: usize, use_uppercase: bool, use_numbers: bool, use_symbols: bool) -> String {
    let mut charset = String::from("abcdefghijklmnopqrstuvwxyz");
    
    if use_uppercase {
        charset.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    }
    if use_numbers {
        charset.push_str("0123456789");
    }
    if use_symbols {
        charset.push_str("!@#$%^&*()_+-=[]{}|;:,.<>?");
    }
    
    let charset_bytes = charset.as_bytes();
    let mut rng = rand::thread_rng();
    
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..charset_bytes.len());
            charset_bytes[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_length() {
        let password = generate_password(12, true, true, true);
        assert_eq!(password.len(), 12);
    }

    #[test]
    fn test_lowercase_only() {
        let password = generate_password(8, false, false, false);
        assert!(password.chars().all(|c| c.is_ascii_lowercase()));
    }

    #[test]
    fn test_character_set_inclusion() {
        let password = generate_password(20, true, true, true);
        assert!(password.chars().any(|c| c.is_ascii_uppercase()));
        assert!(password.chars().any(|c| c.is_ascii_digit()));
        assert!(password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c)));
    }
}use rand::{thread_rng, Rng};
use std::collections::HashSet;

pub struct PasswordGenerator {
    length: usize,
    use_lowercase: bool,
    use_uppercase: bool,
    use_digits: bool,
    use_special: bool,
}

impl PasswordGenerator {
    pub fn new(length: usize) -> Self {
        PasswordGenerator {
            length,
            use_lowercase: true,
            use_uppercase: true,
            use_digits: true,
            use_special: true,
        }
    }

    pub fn lowercase(mut self, enable: bool) -> Self {
        self.use_lowercase = enable;
        self
    }

    pub fn uppercase(mut self, enable: bool) -> Self {
        self.use_uppercase = enable;
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
            return Err("Password length must be greater than zero");
        }

        let mut character_pool = Vec::new();
        
        if self.use_lowercase {
            character_pool.extend(b'a'..=b'z');
        }
        if self.use_uppercase {
            character_pool.extend(b'A'..=b'Z');
        }
        if self.use_digits {
            character_pool.extend(b'0'..=b'9');
        }
        if self.use_special {
            character_pool.extend(b'!'..=b'/');
            character_pool.extend(b':'..=b'@');
            character_pool.extend(b'['..=b'`');
            character_pool.extend(b'{'..=b'~');
        }

        if character_pool.is_empty() {
            return Err("At least one character set must be enabled");
        }

        let mut rng = thread_rng();
        let mut password = String::with_capacity(self.length);
        let mut used_chars = HashSet::new();

        for _ in 0..self.length {
            let idx = rng.gen_range(0..character_pool.len());
            let ch = character_pool[idx] as char;
            password.push(ch);
            used_chars.insert(ch);
        }

        if used_chars.len() < (self.length / 2).max(1) {
            return Err("Generated password lacks sufficient character variety");
        }

        Ok(password)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_password_generation() {
        let generator = PasswordGenerator::new(12);
        let password = generator.generate().unwrap();
        assert_eq!(password.len(), 12);
    }

    #[test]
    fn test_custom_character_sets() {
        let generator = PasswordGenerator::new(8)
            .uppercase(false)
            .special(false);
        let password = generator.generate().unwrap();
        assert!(password.chars().all(|c| c.is_lowercase() || c.is_digit()));
    }

    #[test]
    fn test_invalid_configuration() {
        let generator = PasswordGenerator::new(10)
            .lowercase(false)
            .uppercase(false)
            .digits(false)
            .special(false);
        assert!(generator.generate().is_err());
    }

    #[test]
    fn test_zero_length() {
        let generator = PasswordGenerator::new(0);
        assert!(generator.generate().is_err());
    }
}