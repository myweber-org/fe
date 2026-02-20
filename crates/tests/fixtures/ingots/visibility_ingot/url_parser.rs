use regex::Regex;
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
        let re = Regex::new(r"^(?:https?://)?([^/?#]+)").unwrap();
        re.captures(&self.url)
            .map(|caps| caps[1].to_string())
    }

    pub fn parse_query_params(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        let query_start = self.url.find('?');
        
        if let Some(start) = query_start {
            let query_str = &self.url[start + 1..];
            for pair in query_str.split('&') {
                let parts: Vec<&str> = pair.split('=').collect();
                if parts.len() == 2 {
                    params.insert(
                        parts[0].to_string(),
                        parts[1].to_string()
                    );
                }
            }
        }
        params
    }

    pub fn is_valid_url(&self) -> bool {
        let url_pattern = Regex::new(
            r"^(https?://)?([\w\-]+\.)+[\w\-]+(/[\w\-\./?%&=]*)?$"
        ).unwrap();
        url_pattern.is_match(&self.url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_extraction() {
        let parser = UrlParser::new("https://example.com/path?query=1");
        assert_eq!(parser.extract_domain(), Some("example.com".to_string()));
    }

    #[test]
    fn test_query_parsing() {
        let parser = UrlParser::new("https://api.service.com/search?q=rust&page=2");
        let params = parser.parse_query_params();
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("page"), Some(&"2".to_string()));
    }

    #[test]
    fn test_url_validation() {
        let valid = UrlParser::new("https://valid-domain.org");
        let invalid = UrlParser::new("not-a-valid-url");
        
        assert!(valid.is_valid_url());
        assert!(!invalid.is_valid_url());
    }
}