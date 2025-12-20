
use regex::Regex;
use std::collections::HashSet;

pub struct Validator {
    email_regex: Regex,
    forbidden_usernames: HashSet<String>,
}

impl Validator {
    pub fn new() -> Self {
        let email_pattern = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$";
        let mut forbidden = HashSet::new();
        forbidden.insert("admin".to_string());
        forbidden.insert("root".to_string());
        forbidden.insert("system".to_string());

        Validator {
            email_regex: Regex::new(email_pattern).unwrap(),
            forbidden_usernames: forbidden,
        }
    }

    pub fn validate_email(&self, email: &str) -> bool {
        self.email_regex.is_match(email.trim())
    }

    pub fn validate_username(&self, username: &str) -> bool {
        let name = username.trim();
        !name.is_empty()
            && name.len() >= 3
            && name.len() <= 20
            && name.chars().all(|c| c.is_alphanumeric() || c == '_')
            && !self.forbidden_usernames.contains(&name.to_lowercase())
    }

    pub fn validate_password_strength(&self, password: &str) -> bool {
        let has_upper = password.chars().any(|c| c.is_uppercase());
        let has_lower = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_digit(10));
        let has_special = password.chars().any(|c| "!@#$%^&*".contains(c));

        password.len() >= 8 && has_upper && has_lower && has_digit && has_special
    }

    pub fn sanitize_input(&self, input: &str) -> String {
        input
            .trim()
            .chars()
            .filter(|&c| !c.is_control())
            .collect::<String>()
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
        assert!(!validator.validate_username("admin"));
        assert!(!validator.validate_username("ab"));
    }

    #[test]
    fn test_password_strength() {
        let validator = Validator::new();
        assert!(validator.validate_password_strength("StrongP@ss1"));
        assert!(!validator.validate_password_strength("weak"));
    }
}