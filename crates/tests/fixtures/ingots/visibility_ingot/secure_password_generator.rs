use rand::Rng;
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
        PasswordGenerator {
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
            return Err("Password length must be greater than zero");
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
            return Err("At least one character set must be enabled");
        }

        let mut rng = rand::thread_rng();
        let mut password = String::with_capacity(self.length);
        let mut used_chars = HashSet::new();

        while password.len() < self.length {
            let idx = rng.gen_range(0..character_pool.len());
            let ch = character_pool[idx] as char;
            
            if used_chars.insert(ch) || password.len() >= self.length - 2 {
                password.push(ch);
            }
        }

        Ok(password)
    }

    pub fn validate_strength(password: &str) -> (bool, Vec<&'static str>) {
        let mut issues = Vec::new();
        let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
        let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let has_special = password.chars().any(|c| !c.is_ascii_alphanumeric());

        if password.len() < 8 {
            issues.push("Password must be at least 8 characters long");
        }
        if !has_upper {
            issues.push("Password must contain at least one uppercase letter");
        }
        if !has_lower {
            issues.push("Password must contain at least one lowercase letter");
        }
        if !has_digit {
            issues.push("Password must contain at least one digit");
        }
        if !has_special {
            issues.push("Password must contain at least one special character");
        }

        let is_strong = issues.is_empty();
        (is_strong, issues)
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
    }

    #[test]
    fn test_custom_character_sets() {
        let generator = PasswordGenerator::new(10)
            .uppercase(false)
            .special(false);
        let password = generator.generate().unwrap();
        assert!(!password.chars().any(|c| c.is_ascii_uppercase()));
        assert!(!password.chars().any(|c| !c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_password_strength() {
        let strong_pass = "StrongP@ssw0rd!";
        let (is_strong, issues) = PasswordGenerator::validate_strength(strong_pass);
        assert!(is_strong);
        assert!(issues.is_empty());

        let weak_pass = "weak";
        let (is_strong, issues) = PasswordGenerator::validate_strength(weak_pass);
        assert!(!is_strong);
        assert!(!issues.is_empty());
    }
}