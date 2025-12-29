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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_email() {
        let validator = Validator::new();
        assert!(validator.validate_email("test@example.com"));
        assert!(!validator.validate_email("invalid-email"));
    }

    #[test]
    fn test_valid_username() {
        let validator = Validator::new();
        assert!(validator.validate_username("user_123"));
        assert!(!validator.validate_username("ab"));
    }

    #[test]
    fn test_sanitize_input() {
        let validator = Validator::new();
        assert_eq!(validator.sanitize_input("  hello  "), "hello");
    }
}