use regex::Regex;

pub struct UrlValidator {
    pattern: Regex,
}

impl UrlValidator {
    pub fn new() -> Self {
        let pattern = Regex::new(
            r"^https?://(?:[-\w]+\.)+[-\w]+(?:/[-\w\./?%&=]*)?$"
        ).expect("Invalid regex pattern");
        
        UrlValidator { pattern }
    }

    pub fn is_valid(&self, url: &str) -> bool {
        self.pattern.is_match(url)
    }

    pub fn extract_domain(&self, url: &str) -> Option<String> {
        if !self.is_valid(url) {
            return None;
        }
        
        url.split("://")
            .nth(1)
            .and_then(|s| s.split('/').next())
            .map(|s| s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_urls() {
        let validator = UrlValidator::new();
        assert!(validator.is_valid("https://example.com"));
        assert!(validator.is_valid("http://sub.domain.co.uk/path"));
        assert!(validator.is_valid("https://api.service.io/v1/resource?id=123"));
    }

    #[test]
    fn test_invalid_urls() {
        let validator = UrlValidator::new();
        assert!(!validator.is_valid("not-a-url"));
        assert!(!validator.is_valid("ftp://example.com"));
        assert!(!validator.is_valid("https://"));
    }

    #[test]
    fn test_domain_extraction() {
        let validator = UrlValidator::new();
        assert_eq!(
            validator.extract_domain("https://github.com/rust-lang/rust"),
            Some("github.com".to_string())
        );
        assert_eq!(
            validator.extract_domain("http://localhost:8080/api"),
            Some("localhost:8080".to_string())
        );
    }
}