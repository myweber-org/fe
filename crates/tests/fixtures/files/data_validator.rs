use regex::Regex;
use std::error::Error;

pub struct Validator {
    email_regex: Regex,
    phone_regex: Regex,
}

impl Validator {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Validator {
            email_regex: Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")?,
            phone_regex: Regex::new(r"^\+?[1-9]\d{1,14}$")?,
        })
    }

    pub fn validate_email(&self, email: &str) -> bool {
        self.email_regex.is_match(email)
    }

    pub fn validate_phone(&self, phone: &str) -> bool {
        self.phone_regex.is_match(phone)
    }

    pub fn sanitize_input(&self, input: &str) -> String {
        input.trim().to_string()
    }

    pub fn validate_length(&self, input: &str, min: usize, max: usize) -> bool {
        let len = input.trim().len();
        len >= min && len <= max
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        let validator = Validator::new().unwrap();
        assert!(validator.validate_email("test@example.com"));
        assert!(!validator.validate_email("invalid-email"));
    }

    #[test]
    fn test_phone_validation() {
        let validator = Validator::new().unwrap();
        assert!(validator.validate_phone("+1234567890"));
        assert!(!validator.validate_phone("abc"));
    }

    #[test]
    fn test_sanitization() {
        let validator = Validator::new().unwrap();
        assert_eq!(validator.sanitize_input("  test  "), "test");
    }

    #[test]
    fn test_length_validation() {
        let validator = Validator::new().unwrap();
        assert!(validator.validate_length("hello", 3, 10));
        assert!(!validator.validate_length("hi", 3, 10));
    }
}
use regex::Regex;

pub struct Validator {
    email_regex: Regex,
    username_regex: Regex,
}

impl Validator {
    pub fn new() -> Self {
        Validator {
            email_regex: Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap(),
            username_regex: Regex::new(r"^[a-zA-Z0-9_]{3,20}$").unwrap(),
        }
    }

    pub fn validate_email(&self, email: &str) -> bool {
        self.email_regex.is_match(email)
    }

    pub fn validate_username(&self, username: &str) -> bool {
        self.username_regex.is_match(username)
    }

    pub fn sanitize_input(&self, input: &str) -> String {
        input.trim().to_string()
    }

    pub fn validate_password_strength(&self, password: &str) -> bool {
        let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
        let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let has_special = password.chars().any(|c| "!@#$%^&*".contains(c));
        
        password.len() >= 8 && has_upper && has_lower && has_digit && has_special
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        let validator = Validator::new();
        assert!(validator.validate_email("test@example.com"));
        assert!(!validator.validate_email("invalid-email"));
    }

    #[test]
    fn test_username_validation() {
        let validator = Validator::new();
        assert!(validator.validate_username("valid_user123"));
        assert!(!validator.validate_username("ab"));
    }

    #[test]
    fn test_password_strength() {
        let validator = Validator::new();
        assert!(validator.validate_password_strength("StrongP@ss1"));
        assert!(!validator.validate_password_strength("weak"));
    }
}