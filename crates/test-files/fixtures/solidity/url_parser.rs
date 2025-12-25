
use regex::Regex;
use std::collections::HashSet;

pub struct UrlParser {
    domain_blacklist: HashSet<String>,
}

impl UrlParser {
    pub fn new() -> Self {
        let mut blacklist = HashSet::new();
        blacklist.insert("localhost".to_string());
        blacklist.insert("127.0.0.1".to_string());
        blacklist.insert("::1".to_string());
        blacklist.insert("0.0.0.0".to_string());
        
        UrlParser {
            domain_blacklist: blacklist,
        }
    }

    pub fn extract_domain(&self, url: &str) -> Option<String> {
        let pattern = r"^(?:https?://)?(?:www\.)?([^:/?\s]+)";
        let re = Regex::new(pattern).ok()?;
        
        let captures = re.captures(url)?;
        let domain = captures.get(1)?.as_str().to_lowercase();
        
        if self.domain_blacklist.contains(&domain) {
            return None;
        }
        
        Some(domain)
    }

    pub fn is_valid_url(&self, url: &str) -> bool {
        let url_pattern = r"^https?://[^\s/$.?#].[^\s]*$";
        Regex::new(url_pattern)
            .map(|re| re.is_match(url))
            .unwrap_or(false)
    }

    pub fn normalize_url(&self, url: &str) -> Option<String> {
        if !self.is_valid_url(url) {
            return None;
        }
        
        let domain = self.extract_domain(url)?;
        let normalized = format!("https://{}", domain);
        Some(normalized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_domain() {
        let parser = UrlParser::new();
        
        assert_eq!(
            parser.extract_domain("https://www.example.com/path"),
            Some("example.com".to_string())
        );
        
        assert_eq!(
            parser.extract_domain("http://subdomain.example.co.uk:8080"),
            Some("subdomain.example.co.uk".to_string())
        );
        
        assert_eq!(
            parser.extract_domain("example.com"),
            Some("example.com".to_string())
        );
    }

    #[test]
    fn test_blacklisted_domains() {
        let parser = UrlParser::new();
        
        assert_eq!(parser.extract_domain("http://localhost:3000"), None);
        assert_eq!(parser.extract_domain("https://127.0.0.1/api"), None);
        assert_eq!(parser.extract_domain("http://0.0.0.0"), None);
    }

    #[test]
    fn test_url_validation() {
        let parser = UrlParser::new();
        
        assert!(parser.is_valid_url("https://example.com"));
        assert!(parser.is_valid_url("http://sub.example.com/path?query=1"));
        assert!(!parser.is_valid_url("not-a-url"));
        assert!(!parser.is_valid_url("ftp://example.com"));
    }

    #[test]
    fn test_normalization() {
        let parser = UrlParser::new();
        
        assert_eq!(
            parser.normalize_url("http://www.example.com"),
            Some("https://example.com".to_string())
        );
        
        assert_eq!(
            parser.normalize_url("https://EXAMPLE.COM/path"),
            Some("https://example.com".to_string())
        );
        
        assert_eq!(parser.normalize_url("invalid-url"), None);
    }
}