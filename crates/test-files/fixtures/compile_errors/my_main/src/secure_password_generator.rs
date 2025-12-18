use rand::Rng;
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

    pub fn generate(&self) -> Result<String, String> {
        let mut character_pool = Vec::new();
        
        if self.use_lowercase {
            character_pool.extend('a'..='z');
        }
        if self.use_uppercase {
            character_pool.extend('A'..='Z');
        }
        if self.use_digits {
            character_pool.extend('0'..='9');
        }
        if self.use_special {
            character_pool.extend("!@#$%^&*()-_=+[]{}|;:,.<>?".chars());
        }

        if character_pool.is_empty() {
            return Err("At least one character set must be enabled".to_string());
        }

        if self.length < 8 {
            return Err("Password length must be at least 8 characters".to_string());
        }

        let mut rng = rand::thread_rng();
        let mut password_chars = Vec::new();
        let mut used_categories = HashSet::new();

        while password_chars.len() < self.length {
            let idx = rng.gen_range(0..character_pool.len());
            let ch = character_pool[idx];
            
            password_chars.push(ch);
            
            if ch.is_ascii_lowercase() {
                used_categories.insert("lowercase");
            } else if ch.is_ascii_uppercase() {
                used_categories.insert("uppercase");
            } else if ch.is_ascii_digit() {
                used_categories.insert("digits");
            } else {
                used_categories.insert("special");
            }
        }

        let required_categories = vec![
            (self.use_lowercase, "lowercase"),
            (self.use_uppercase, "uppercase"),
            (self.use_digits, "digits"),
            (self.use_special, "special"),
        ];

        for (enabled, category) in required_categories {
            if enabled && !used_categories.contains(category) {
                return self.generate();
            }
        }

        Ok(password_chars.into_iter().collect())
    }
}

pub fn generate_password(length: usize) -> Result<String, String> {
    PasswordGenerator::new(length).generate()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_length() {
        let password = generate_password(12).unwrap();
        assert_eq!(password.len(), 12);
    }

    #[test]
    fn test_password_contains_all_categories() {
        let password = generate_password(16).unwrap();
        assert!(password.chars().any(|c| c.is_ascii_lowercase()));
        assert!(password.chars().any(|c| c.is_ascii_uppercase()));
        assert!(password.chars().any(|c| c.is_ascii_digit()));
        assert!(password.chars().any(|c| !c.is_alphanumeric()));
    }

    #[test]
    fn test_custom_configuration() {
        let generator = PasswordGenerator::new(10)
            .uppercase(false)
            .special(false);
        
        let password = generator.generate().unwrap();
        assert!(!password.chars().any(|c| c.is_ascii_uppercase()));
        assert!(!password.chars().any(|c| !c.is_alphanumeric()));
    }

    #[test]
    fn test_invalid_length() {
        let result = generate_password(5);
        assert!(result.is_err());
    }

    #[test]
    fn test_no_character_sets() {
        let generator = PasswordGenerator::new(10)
            .lowercase(false)
            .uppercase(false)
            .digits(false)
            .special(false);
        
        let result = generator.generate();
        assert!(result.is_err());
    }
}