
use regex::Regex;
use std::error::Error;

#[derive(Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
}

pub struct DataValidator {
    email_regex: Regex,
    username_regex: Regex,
}

impl DataValidator {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(DataValidator {
            email_regex: Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")?,
            username_regex: Regex::new(r"^[a-zA-Z0-9_]{3,20}$")?,
        })
    }

    pub fn validate_email(&self, email: &str) -> ValidationResult {
        let mut errors = Vec::new();
        
        if email.is_empty() {
            errors.push("Email cannot be empty".to_string());
        }
        
        if !self.email_regex.is_match(email) {
            errors.push("Invalid email format".to_string());
        }
        
        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
        }
    }

    pub fn validate_username(&self, username: &str) -> ValidationResult {
        let mut errors = Vec::new();
        
        if username.len() < 3 {
            errors.push("Username must be at least 3 characters".to_string());
        }
        
        if username.len() > 20 {
            errors.push("Username cannot exceed 20 characters".to_string());
        }
        
        if !self.username_regex.is_match(username) {
            errors.push("Username can only contain letters, numbers and underscores".to_string());
        }
        
        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
        }
    }

    pub fn validate_password(&self, password: &str) -> ValidationResult {
        let mut errors = Vec::new();
        let has_uppercase = password.chars().any(|c| c.is_ascii_uppercase());
        let has_lowercase = password.chars().any(|c| c.is_ascii_lowercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        
        if password.len() < 8 {
            errors.push("Password must be at least 8 characters".to_string());
        }
        
        if !has_uppercase {
            errors.push("Password must contain at least one uppercase letter".to_string());
        }
        
        if !has_lowercase {
            errors.push("Password must contain at least one lowercase letter".to_string());
        }
        
        if !has_digit {
            errors.push("Password must contain at least one digit".to_string());
        }
        
        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_email() {
        let validator = DataValidator::new().unwrap();
        let result = validator.validate_email("test@example.com");
        assert!(result.is_valid);
    }

    #[test]
    fn test_invalid_email() {
        let validator = DataValidator::new().unwrap();
        let result = validator.validate_email("invalid-email");
        assert!(!result.is_valid);
    }

    #[test]
    fn test_valid_username() {
        let validator = DataValidator::new().unwrap();
        let result = validator.validate_username("valid_user123");
        assert!(result.is_valid);
    }

    #[test]
    fn test_strong_password() {
        let validator = DataValidator::new().unwrap();
        let result = validator.validate_password("StrongPass123");
        assert!(result.is_valid);
    }
}