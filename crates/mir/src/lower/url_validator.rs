use regex::Regex;

pub struct UrlValidator {
    pattern: Regex,
}

impl UrlValidator {
    pub fn new() -> Self {
        let pattern = Regex::new(r"^https?://(?:[-\w]+\.)+[-\w]{2,}(?:/[\w\-./?%&=]*)?$").unwrap();
        UrlValidator { pattern }
    }

    pub fn is_valid(&self, url: &str) -> bool {
        self.pattern.is_match(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_urls() {
        let validator = UrlValidator::new();
        assert!(validator.is_valid("https://example.com"));
        assert!(validator.is_valid("http://sub.example.com/path"));
        assert!(validator.is_valid("https://api.service.io/v1/resource?id=123"));
    }

    #[test]
    fn test_invalid_urls() {
        let validator = UrlValidator::new();
        assert!(!validator.is_valid("not-a-url"));
        assert!(!validator.is_valid("ftp://example.com"));
        assert!(!validator.is_valid("https://"));
    }
}