use regex::Regex;

pub fn is_valid_email(email: &str) -> bool {
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    email_regex.is_match(email)
}

pub fn is_valid_phone(phone: &str) -> bool {
    let phone_regex = Regex::new(r"^\+?[1-9]\d{1,14}$").unwrap();
    phone_regex.is_match(phone)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_emails() {
        assert!(is_valid_email("user@example.com"));
        assert!(is_valid_email("john.doe@company.co.uk"));
        assert!(is_valid_email("alice+tag@domain.org"));
    }

    #[test]
    fn test_invalid_emails() {
        assert!(!is_valid_email("invalid-email"));
        assert!(!is_valid_email("user@.com"));
        assert!(!is_valid_email("@domain.com"));
    }

    #[test]
    fn test_valid_phones() {
        assert!(is_valid_phone("+1234567890"));
        assert!(is_valid_phone("1234567890"));
        assert!(is_valid_phone("+441234567890"));
    }

    #[test]
    fn test_invalid_phones() {
        assert!(!is_valid_phone("123"));
        assert!(!is_valid_phone("+01234567890"));
        assert!(!is_valid_phone("abc1234567"));
    }
}