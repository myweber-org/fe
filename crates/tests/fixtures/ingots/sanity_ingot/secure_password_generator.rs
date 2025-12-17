
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

        for &required_char in &required_chars {
            password_chars.push(required_char as char);
        }

        while password_chars.len() < self.length {
            let random_index = rng.gen_range(0..character_pool.len());
            password_chars.push(character_pool[random_index] as char);
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

    fn shuffle<T>(slice: &mut [T]) {
        let mut rng = rand::thread_rng();
        for i in (1..slice.len()).rev() {
            let j = rng.gen_range(0..=i);
            slice.swap(i, j);
        }
    }
}

pub fn validate_password_strength(password: &str) -> (bool, HashSet<&'static str>) {
    let mut issues = HashSet::new();
    
    if password.len() < 8 {
        issues.insert("Password must be at least 8 characters long");
    }
    
    let mut has_lowercase = false;
    let mut has_uppercase = false;
    let mut has_digit = false;
    let mut has_special = false;
    
    for ch in password.chars() {
        if ch.is_ascii_lowercase() {
            has_lowercase = true;
        } else if ch.is_ascii_uppercase() {
            has_uppercase = true;
        } else if ch.is_ascii_digit() {
            has_digit = true;
        } else if ch.is_ascii_punctuation() {
            has_special = true;
        }
    }
    
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