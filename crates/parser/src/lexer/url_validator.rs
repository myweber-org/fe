use regex::Regex;

pub struct UrlValidator {
    pattern: Regex,
}

impl UrlValidator {
    pub fn new() -> Self {
        let pattern = Regex::new(
            r"^(https?|ftp)://[^\s/$.?#].[^\s]*$"
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
        
        let domain_pattern = Regex::new(
            r"^(?:https?://)?([^/:]+)"
        ).expect("Invalid domain regex");
        
        domain_pattern.captures(url)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
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
        assert!(validator.is_valid("ftp://files.server.org"));
    }

    #[test]
    fn test_invalid_urls() {
        let validator = UrlValidator::new();
        assert!(!validator.is_valid("not-a-url"));
        assert!(!validator.is_valid("http://"));
        assert!(!validator.is_valid("example.com"));
    }

    #[test]
    fn test_domain_extraction() {
        let validator = UrlValidator::new();
        assert_eq!(
            validator.extract_domain("https://www.github.com/rust-lang"),
            Some("www.github.com".to_string())
        );
        assert_eq!(
            validator.extract_domain("http://localhost:8080"),
            Some("localhost".to_string())
        );
    }
}