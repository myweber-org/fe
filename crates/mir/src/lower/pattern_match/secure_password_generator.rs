
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
            return Err("Password length must be greater than 0");
        }

        let mut character_set = Vec::new();
        
        if self.use_lowercase {
            character_set.extend(b'a'..=b'z');
        }
        if self.use_uppercase {
            character_set.extend(b'A'..=b'Z');
        }
        if self.use_digits {
            character_set.extend(b'0'..=b'9');
        }
        if self.use_special {
            character_set.extend(b'!'..=b'/');
            character_set.extend(b':'..=b'@');
            character_set.extend(b'['..=b'`');
            character_set.extend(b'{'..=b'~');
        }

        if character_set.is_empty() {
            return Err("At least one character set must be enabled");
        }

        let mut rng = rand::thread_rng();
        let mut password = String::with_capacity(self.length);
        let mut used_chars = HashSet::new();

        for _ in 0..self.length {
            let idx = rng.gen_range(0..character_set.len());
            let ch = character_set[idx] as char;
            password.push(ch);
            used_chars.insert(ch);
        }

        Ok(password)
    }

    pub fn generate_multiple(&self, count: usize) -> Result<Vec<String>, &'static str> {
        let mut passwords = Vec::with_capacity(count);
        for _ in 0..count {
            passwords.push(self.generate()?);
        }
        Ok(passwords)
    }
}

pub fn validate_password_strength(password: &str) -> bool {
    if password.len() < 8 {
        return false;
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

    has_lowercase && has_uppercase && has_digit && has_special
}