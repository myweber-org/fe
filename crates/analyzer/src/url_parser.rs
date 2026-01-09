
use std::collections::HashMap;

pub struct UrlParser {
    url: String,
}

impl UrlParser {
    pub fn new(url: &str) -> Self {
        UrlParser {
            url: url.to_string(),
        }
    }

    pub fn extract_domain(&self) -> Option<String> {
        let url = self.url.trim();
        if url.is_empty() {
            return None;
        }

        let url_lower = url.to_lowercase();
        let prefixes = ["http://", "https://", "www."];
        
        let mut domain_start = 0;
        for prefix in prefixes.iter() {
            if url_lower.starts_with(prefix) {
                domain_start = prefix.len();
                break;
            }
        }

        let remaining = &url[domain_start..];
        let domain_end = remaining.find('/').unwrap_or(remaining.len());
        let domain = &remaining[..domain_end];

        if domain.is_empty() {
            None
        } else {
            Some(domain.to_string())
        }
    }

    pub fn parse_query_params(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if let Some(query_start) = self.url.find('?') {
            let query_string = &self.url[query_start + 1..];
            
            for pair in query_string.split('&') {
                let parts: Vec<&str> = pair.split('=').collect();
                if parts.len() == 2 {
                    let key = parts[0].to_string();
                    let value = parts[1].to_string();
                    params.insert(key, value);
                }
            }
        }
        
        params
    }

    pub fn is_valid_url(&self) -> bool {
        let url = self.url.trim();
        if url.is_empty() {
            return false;
        }

        let url_lower = url.to_lowercase();
        let has_protocol = url_lower.starts_with("http://") || url_lower.starts_with("https://");
        let has_dot = url.contains('.');
        
        has_protocol && has_dot && url.len() > 10
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_extraction() {
        let parser = UrlParser::new("https://www.example.com/path");
        assert_eq!(parser.extract_domain(), Some("www.example.com".to_string()));
        
        let parser2 = UrlParser::new("http://api.github.com/users");
        assert_eq!(parser2.extract_domain(), Some("api.github.com".to_string()));
    }

    #[test]
    fn test_query_parsing() {
        let parser = UrlParser::new("https://example.com/search?q=rust&sort=desc");
        let params = parser.parse_query_params();
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("sort"), Some(&"desc".to_string()));
    }

    #[test]
    fn test_url_validation() {
        let valid_parser = UrlParser::new("https://example.com");
        assert!(valid_parser.is_valid_url());
        
        let invalid_parser = UrlParser::new("not-a-url");
        assert!(!invalid_parser.is_valid_url());
    }
}