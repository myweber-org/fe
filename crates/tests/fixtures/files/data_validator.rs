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