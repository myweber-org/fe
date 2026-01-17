use regex::Regex;
use std::error::Error;

pub struct Validator {
    email_regex: Regex,
    username_regex: Regex,
}

impl Validator {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Validator {
            email_regex: Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")?,
            username_regex: Regex::new(r"^[a-zA-Z0-9_]{3,20}$")?,
        })
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
    fn test_username_validation() {
        let validator = Validator::new().unwrap();
        assert!(validator.validate_username("valid_user123"));
        assert!(!validator.validate_username("ab"));
    }

    #[test]
    fn test_sanitization() {
        let validator = Validator::new().unwrap();
        assert_eq!(validator.sanitize_input("  test  "), "test");
    }
}