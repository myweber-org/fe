use regex::Regex;

pub struct UrlValidator {
    pattern: Regex,
}

impl UrlValidator {
    pub fn new() -> Self {
        let pattern = Regex::new(r"^https?://(?:www\.)?[a-zA-Z0-9-]+\.[a-zA-Z]{2,}(?:/[^\s]*)?$")
            .expect("Invalid regex pattern");
        UrlValidator { pattern }
    }

    pub fn is_valid(&self, url: &str) -> bool {
        self.pattern.is_match(url)
    }

    pub fn extract_domain(&self, url: &str) -> Option<String> {
        if !self.is_valid(url) {
            return None;
        }
        
        let domain_start = url.find("://").map(|i| i + 3).unwrap_or(0);
        let domain_end = url[domain_start..]
            .find('/')
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
        let validator = UrlValidator::new();
        assert!(validator.is_valid("https://example.com"));
        assert!(validator.is_valid("http://sub.example.com/path"));
        assert!(validator.is_valid("https://www.example.co.uk"));
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
            validator.extract_domain("https://example.com/path"),
            Some("example.com".to_string())
        );
        assert_eq!(
            validator.extract_domain("http://www.google.com"),
            Some("www.google.com".to_string())
        );
        assert!(validator.extract_domain("invalid-url").is_none());
    }
}