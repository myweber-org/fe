
use rand::Rng;

pub struct PasswordGenerator {
    length: usize,
    use_lowercase: bool,
    use_uppercase: bool,
    use_digits: bool,
    use_special: bool,
}

impl PasswordGenerator {
    pub fn new(length: usize) -> Self {
        Self {
            length,
            use_lowercase: true,
            use_uppercase: true,
            use_digits: true,
            use_special: true,
        }
    }

    pub fn with_lowercase(mut self, enable: bool) -> Self {
        self.use_lowercase = enable;
        self
    }

    pub fn with_uppercase(mut self, enable: bool) -> Self {
        self.use_uppercase = enable;
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

    pub fn generate(&self) -> Result<String, &'static str> {
        if self.length == 0 {
            return Err("Password length must be greater than 0");
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

        let mut rng = rand::thread_rng();
        let password: String = (0..self.length)
            .map(|_| {
                let idx = rng.gen_range(0..character_pool.len());
                character_pool[idx] as char
            })
            .collect();

        Ok(password)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_password() {
        let generator = PasswordGenerator::new(12);
        let password = generator.generate().unwrap();
        assert_eq!(password.len(), 12);
    }

    #[test]
    fn test_custom_character_sets() {
        let generator = PasswordGenerator::new(8)
            .with_lowercase(true)
            .with_uppercase(false)
            .with_digits(false)
            .with_special(false);
        
        let password = generator.generate().unwrap();
        assert!(password.chars().all(|c| c.is_lowercase()));
    }

    #[test]
    fn test_empty_character_set() {
        let generator = PasswordGenerator::new(10)
            .with_lowercase(false)
            .with_uppercase(false)
            .with_digits(false)
            .with_special(false);
        
        let result = generator.generate();
        assert!(result.is_err());
    }

    #[test]
    fn test_zero_length() {
        let generator = PasswordGenerator::new(0);
        let result = generator.generate();
        assert!(result.is_err());
    }
}use rand::Rng;
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
        Self {
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
        let mut required_chars = Vec::new();

        if self.use_lowercase {
            character_pool.extend(b'a'..=b'z');
            required_chars.push(self.random_char_from_range(b'a'..=b'z'));
        }

        if self.use_uppercase {
            character_pool.extend(b'A'..=b'Z');
            required_chars.push(self.random_char_from_range(b'A'..=b'Z'));
        }

        if self.use_digits {
            character_pool.extend(b'0'..=b'9');
            required_chars.push(self.random_char_from_range(b'0'..=b'9'));
        }

        if self.use_special {
            let special_chars = b"!@#$%^&*()_+-=[]{}|;:,.<>?";
            character_pool.extend_from_slice(special_chars);
            required_chars.push(self.random_char_from_slice(special_chars));
        }

        if character_pool.is_empty() {
            return Err("At least one character set must be enabled");
        }

        let mut rng = rand::thread_rng();
        let mut password_chars: Vec<char> = Vec::with_capacity(self.length);

        for &ch in &required_chars {
            password_chars.push(ch as char);
        }

        while password_chars.len() < self.length {
            let idx = rng.gen_range(0..character_pool.len());
            password_chars.push(character_pool[idx] as char);
        }

        Self::shuffle(&mut password_chars);
        Ok(password_chars.into_iter().collect())
    }

    fn random_char_from_range<R: rand::distributions::uniform::SampleRange<u8>>(&self, range: R) -> u8 {
        let mut rng = rand::thread_rng();
        rng.gen_range(range)
    }

    fn random_char_from_slice(&self, slice: &[u8]) -> u8 {
        let mut rng = rand::thread_rng();
        slice[rng.gen_range(0..slice.len())]
    }

    fn shuffle<T>(items: &mut [T]) {
        let mut rng = rand::thread_rng();
        for i in (1..items.len()).rev() {
            let j = rng.gen_range(0..=i);
            items.swap(i, j);
        }
    }
}

pub fn validate_password_strength(password: &str) -> (bool, HashSet<&'static str>) {
    let mut issues = HashSet::new();
    
    if password.len() < 8 {
        issues.insert("Password must be at least 8 characters long");
    }
    
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_digit = password.chars().any(|c| c.is_digit(10));
    let has_special = password.chars().any(|c| !c.is_alphanumeric());
    
    if !has_lowercase {
        issues.insert("Password must contain at least one lowercase letter");
    }
    
    if !has_uppercase {
        issues.insert("Password must contain at least one uppercase letter");
    }
    
    if !has_digit {
        issues.insert("Password must contain at least one digit");
    }
    
    if !has_special {
        issues.insert("Password must contain at least one special character");
    }
    
    (issues.is_empty(), issues)
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
        let validation = validate_password_strength(&password);
        
        assert!(validation.1.contains("Password must contain at least one uppercase letter"));
        assert!(validation.1.contains("Password must contain at least one special character"));
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