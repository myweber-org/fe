use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

pub struct PasswordGenerator {
    length: usize,
    use_uppercase: bool,
    use_numbers: bool,
    use_special: bool,
}

impl PasswordGenerator {
    pub fn new(length: usize) -> Self {
        PasswordGenerator {
            length,
            use_uppercase: true,
            use_numbers: true,
            use_special: true,
        }
    }

    pub fn uppercase(mut self, enable: bool) -> Self {
        self.use_uppercase = enable;
        self
    }

    pub fn numbers(mut self, enable: bool) -> Self {
        self.use_numbers = enable;
        self
    }

    pub fn special(mut self, enable: bool) -> Self {
        self.use_special = enable;
        self
    }

    pub fn generate(&self) -> String {
        let mut charset = String::from("abcdefghijklmnopqrstuvwxyz");
        
        if self.use_uppercase {
            charset.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
        }
        
        if self.use_numbers {
            charset.push_str("0123456789");
        }
        
        if self.use_special {
            charset.push_str("!@#$%^&*()_+-=[]{}|;:,.<>?");
        }

        let mut rng = thread_rng();
        (0..self.length)
            .map(|_| {
                let idx = rng.gen_range(0..charset.len());
                charset.chars().nth(idx).unwrap()
            })
            .collect()
    }
}

pub fn generate_alphanumeric(length: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_length() {
        let generator = PasswordGenerator::new(12);
        let password = generator.generate();
        assert_eq!(password.len(), 12);
    }

    #[test]
    fn test_alphanumeric_generator() {
        let password = generate_alphanumeric(16);
        assert_eq!(password.len(), 16);
        assert!(password.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_custom_charset() {
        let generator = PasswordGenerator::new(10)
            .uppercase(false)
            .numbers(false)
            .special(false);
        
        let password = generator.generate();
        assert!(password.chars().all(|c| c.is_ascii_lowercase()));
    }
}