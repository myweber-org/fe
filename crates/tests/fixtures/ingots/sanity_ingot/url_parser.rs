
use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_query_params(url: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if let Some(query_start) = url.find('?') {
            let query_string = &url[query_start + 1..];
            
            for param_pair in query_string.split('&') {
                let mut parts = param_pair.split('=');
                if let Some(key) = parts.next() {
                    if let Some(value) = parts.next() {
                        params.insert(key.to_string(), value.to_string());
                    }
                }
            }
        }
        
        params
    }
    
    pub fn extract_domain(url: &str) -> Option<String> {
        let url_lower = url.to_lowercase();
        
        if url_lower.starts_with("http://") || url_lower.starts_with("https://") {
            let after_protocol = &url[url.find("://").unwrap() + 3..];
            if let Some(slash_pos) = after_protocol.find('/') {
                return Some(after_protocol[..slash_pos].to_string());
            }
            return Some(after_protocol.to_string());
        }
        
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_query_params() {
        let url = "https://example.com/search?q=rust&lang=en&sort=date";
        let params = UrlParser::parse_query_params(url);
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("lang"), Some(&"en".to_string()));
        assert_eq!(params.get("sort"), Some(&"date".to_string()));
        assert_eq!(params.len(), 3);
    }
    
    #[test]
    fn test_extract_domain() {
        assert_eq!(
            UrlParser::extract_domain("https://api.github.com/users"),
            Some("api.github.com".to_string())
        );
        
        assert_eq!(
            UrlParser::extract_domain("http://localhost:8080/path"),
            Some("localhost:8080".to_string())
        );
        
        assert_eq!(
            UrlParser::extract_domain("invalid-url"),
            None
        );
    }
    
    #[test]
    fn test_empty_query() {
        let url = "https://example.com/page";
        let params = UrlParser::parse_query_params(url);
        assert!(params.is_empty());
    }
}