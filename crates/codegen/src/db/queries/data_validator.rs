use regex::Regex;

pub fn validate_email(email: &str) -> bool {
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    email_regex.is_match(email)
}

pub fn validate_phone(phone: &str) -> bool {
    let phone_regex = Regex::new(r"^\+?[1-9]\d{1,14}$").unwrap();
    phone_regex.is_match(phone)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_emails() {
        assert!(validate_email("user@example.com"));
        assert!(validate_email("john.doe@company.co.uk"));
        assert!(validate_email("alice+test@domain.org"));
    }

    #[test]
    fn test_invalid_emails() {
        assert!(!validate_email("invalid-email"));
        assert!(!validate_email("user@.com"));
        assert!(!validate_email("@domain.com"));
    }

    #[test]
    fn test_valid_phones() {
        assert!(validate_phone("+12345678901"));
        assert!(validate_phone("1234567890"));
        assert!(validate_phone("+441234567890"));
    }

    #[test]
    fn test_invalid_phones() {
        assert!(!validate_phone("abc"));
        assert!(!validate_phone("123"));
        assert!(!validate_phone("+0123456789"));
    }
}