
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
            "admin", "root", "system", "administrator", 
            "moderator", "support", "test"
        ].into_iter().map(String::from).collect();

        Validator {
            email_regex: Regex::new(email_pattern).unwrap(),
            forbidden_usernames: forbidden,
        }
    }

    pub fn validate_email(&self, email: &str) -> bool {
        self.email_regex.is_match(email.trim())
    }

    pub fn validate_username(&self, username: &str) -> Result<(), String> {
        let name = username.trim();
        
        if name.len() < 3 || name.len() > 20 {
            return Err("Username must be between 3 and 20 characters".to_string());
        }
        
        if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err("Username can only contain alphanumeric characters, underscores and hyphens".to_string());
        }
        
        if self.forbidden_usernames.contains(&name.to_lowercase()) {
            return Err("This username is not allowed".to_string());
        }
        
        Ok(())
    }

    pub fn sanitize_input(&self, input: &str) -> String {
        input.trim()
            .chars()
            .filter(|&c| !c.is_control())
            .collect()
    }

    pub fn validate_password_strength(&self, password: &str) -> Result<(), String> {
        if password.len() < 8 {
            return Err("Password must be at least 8 characters long".to_string());
        }
        
        let has_upper = password.chars().any(|c| c.is_uppercase());
        let has_lower = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_digit(10));
        let has_special = password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c));
        
        let score = [has_upper, has_lower, has_digit, has_special]
            .iter()
            .filter(|&&x| x)
            .count();
        
        if score < 3 {
            return Err("Password must contain at least three of: uppercase, lowercase, digits, special characters".to_string());
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        let validator = Validator::new();
        assert!(validator.validate_email("user@example.com"));
        assert!(validator.validate_email("test.user+tag@domain.co.uk"));
        assert!(!validator.validate_email("invalid-email"));
        assert!(!validator.validate_email("user@.com"));
    }

    #[test]
    fn test_username_validation() {
        let validator = Validator::new();
        assert!(validator.validate_username("valid_user-123").is_ok());
        assert!(validator.validate_username("ab").is_err());
        assert!(validator.validate_username("admin").is_err());
        assert!(validator.validate_username("user name").is_err());
    }

    #[test]
    fn test_password_strength() {
        let validator = Validator::new();
        assert!(validator.validate_password_strength("StrongPass123!").is_ok());
        assert!(validator.validate_password_strength("weak").is_err());
        assert!(validator.validate_password_strength("NoSpecial123").is_ok());
        assert!(validator.validate_password_strength("alllowercase!").is_err());
    }

    #[test]
    fn test_sanitize_input() {
        let validator = Validator::new();
        let input = "  Hello\tWorld\n";
        assert_eq!(validator.sanitize_input(input), "HelloWorld");
    }
}
use regex::Regex;

pub fn is_valid_email(email: &str) -> bool {
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    email_regex.is_match(email)
}

pub fn is_valid_phone(phone: &str) -> bool {
    let phone_regex = Regex::new(r"^\+?[1-9]\d{1,14}$").unwrap();
    phone_regex.is_match(phone)
}

pub fn validate_user_data(email: &str, phone: &str) -> Result<(), String> {
    if !is_valid_email(email) {
        return Err(format!("Invalid email address: {}", email));
    }
    
    if !is_valid_phone(phone) {
        return Err(format!("Invalid phone number: {}", phone));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_email() {
        assert!(is_valid_email("user@example.com"));
        assert!(is_valid_email("john.doe@company.co.uk"));
        assert!(!is_valid_email("invalid-email"));
        assert!(!is_valid_email("user@.com"));
    }

    #[test]
    fn test_valid_phone() {
        assert!(is_valid_phone("+1234567890"));
        assert!(is_valid_phone("1234567890"));
        assert!(!is_valid_phone("abc123"));
        assert!(!is_valid_phone("123"));
    }

    #[test]
    fn test_validate_user_data() {
        assert!(validate_user_data("test@example.com", "+1234567890").is_ok());
        assert!(validate_user_data("invalid", "+1234567890").is_err());
        assert!(validate_user_data("test@example.com", "invalid").is_err());
    }
}