
use regex::Regex;
use std::collections::HashSet;

pub struct Validator {
    email_regex: Regex,
    forbidden_usernames: HashSet<String>,
}

impl Validator {
    pub fn new() -> Self {
        let email_pattern = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$";
        let forbidden = vec![
            "admin".to_string(),
            "root".to_string(),
            "system".to_string(),
            "test".to_string(),
        ]
        .into_iter()
        .collect();

        Validator {
            email_regex: Regex::new(email_pattern).unwrap(),
            forbidden_usernames: forbidden,
        }
    }

    pub fn validate_email(&self, email: &str) -> bool {
        self.email_regex.is_match(email)
    }

    pub fn validate_username(&self, username: &str) -> Result<(), String> {
        if username.len() < 3 {
            return Err("Username must be at least 3 characters".to_string());
        }

        if username.len() > 20 {
            return Err("Username must not exceed 20 characters".to_string());
        }

        if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err("Username can only contain alphanumeric characters and underscores".to_string());
        }

        if self.forbidden_usernames.contains(username) {
            return Err("This username is not allowed".to_string());
        }

        Ok(())
    }

    pub fn validate_password_strength(&self, password: &str) -> bool {
        let has_upper = password.chars().any(|c| c.is_uppercase());
        let has_lower = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_digit(10));
        let has_special = password.chars().any(|c| !c.is_alphanumeric());

        password.len() >= 8 && has_upper && has_lower && has_digit && has_special
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        let validator = Validator::new();
        assert!(validator.validate_email("user@example.com"));
        assert!(!validator.validate_email("invalid-email"));
    }

    #[test]
    fn test_username_validation() {
        let validator = Validator::new();
        assert!(validator.validate_username("valid_user").is_ok());
        assert!(validator.validate_username("admin").is_err());
    }

    #[test]
    fn test_password_strength() {
        let validator = Validator::new();
        assert!(validator.validate_password_strength("StrongPass123!"));
        assert!(!validator.validate_password_strength("weak"));
    }
}