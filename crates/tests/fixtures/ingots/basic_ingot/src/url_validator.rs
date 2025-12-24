use regex::Regex;
use std::error::Error;

pub struct UrlValidator {
    pattern: Regex,
}

impl UrlValidator {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let pattern = Regex::new(r"^https?://(?:www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b(?:[-a-zA-Z0-9()@:%_\+.~#?&//=]*)$")?;
        Ok(UrlValidator { pattern })
    }

    pub fn validate(&self, url: &str) -> bool {
        self.pattern.is_match(url)
    }

    pub fn extract_domain(&self, url: &str) -> Option<String> {
        if !self.validate(url) {
            return None;
        }
        
        let domain_start = url.find("://").map(|i| i + 3).unwrap_or(0);
        let domain_end = url[domain_start..]
            .find(|c| c == '/' || c == '?' || c == '#')
            .map(|i| domain_start + i)
            .unwrap_or(url.len());
            
        Some(url[domain_start..domain_end].to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_urls() {
        let validator = UrlValidator::new().unwrap();
        assert!(validator.validate("https://example.com"));
        assert!(validator.validate("http://sub.example.co.uk/path?query=value"));
        assert!(validator.validate("https://www.google.com/search?q=rust"));
    }

    #[test]
    fn test_invalid_urls() {
        let validator = UrlValidator::new().unwrap();
        assert!(!validator.validate("not-a-url"));
        assert!(!validator.validate("ftp://example.com"));
        assert!(!validator.validate("://missing-protocol.com"));
    }

    #[test]
    fn test_domain_extraction() {
        let validator = UrlValidator::new().unwrap();
        assert_eq!(validator.extract_domain("https://example.com/path"), Some("example.com".to_string()));
        assert_eq!(validator.extract_domain("http://sub.domain.co.uk"), Some("sub.domain.co.uk".to_string()));
        assert_eq!(validator.extract_domain("invalid-url"), None);
    }
}