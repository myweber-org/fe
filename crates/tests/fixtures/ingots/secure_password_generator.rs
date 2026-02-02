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
        let mut required_sets = Vec::new();

        if self.use_lowercase {
            let set: Vec<char> = ('a'..='z').collect();
            character_pool.extend(&set);
            required_sets.push(set);
        }

        if self.use_uppercase {
            let set: Vec<char> = ('A'..='Z').collect();
            character_pool.extend(&set);
            required_sets.push(set);
        }

        if self.use_digits {
            let set: Vec<char> = ('0'..='9').collect();
            character_pool.extend(&set);
            required_sets.push(set);
        }

        if self.use_special {
            let set: Vec<char> = "!@#$%^&*()-_=+[]{}|;:,.<>?".chars().collect();
            character_pool.extend(&set);
            required_sets.push(set);
        }

        if character_pool.is_empty() {
            return Err("At least one character set must be enabled");
        }

        let mut rng = rand::thread_rng();
        let mut password_chars = Vec::with_capacity(self.length);
        let mut used_sets = HashSet::new();

        for i in 0..self.length {
            if i < required_sets.len() {
                let set_index = i % required_sets.len();
                let char_set = &required_sets[set_index];
                let random_char = char_set[rng.gen_range(0..char_set.len())];
                password_chars.push(random_char);
                used_sets.insert(set_index);
            } else {
                let random_index = rng.gen_range(0..character_pool.len());
                password_chars.push(character_pool[random_index]);
            }
        }

        for (i, set) in required_sets.iter().enumerate() {
            if !used_sets.contains(&i) {
                let random_char = set[rng.gen_range(0..set.len())];
                let replace_index = rng.gen_range(0..self.length);
                password_chars[replace_index] = random_char;
            }
        }

        rng.shuffle(&mut password_chars);

        Ok(password_chars.into_iter().collect())
    }
}

pub fn generate_password(length: usize) -> Result<String, &'static str> {
    PasswordGenerator::new(length).generate()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_generator() {
        let password = generate_password(12).unwrap();
        assert_eq!(password.len(), 12);
        assert!(password.chars().any(|c| c.is_lowercase()));
        assert!(password.chars().any(|c| c.is_uppercase()));
        assert!(password.chars().any(|c| c.is_digit(10)));
        assert!(password.chars().any(|c| "!@#$%^&*()-_=+[]{}|;:,.<>?".contains(c)));
    }

    #[test]
    fn test_custom_generator() {
        let generator = PasswordGenerator::new(16)
            .uppercase(false)
            .special(false);
        
        let password = generator.generate().unwrap();
        assert_eq!(password.len(), 16);
        assert!(password.chars().any(|c| c.is_lowercase()));
        assert!(password.chars().any(|c| c.is_digit(10)));
        assert!(!password.chars().any(|c| c.is_uppercase()));
        assert!(!password.chars().any(|c| "!@#$%^&*()-_=+[]{}|;:,.<>?".contains(c)));
    }

    #[test]
    fn test_invalid_length() {
        let result = generate_password(0);
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