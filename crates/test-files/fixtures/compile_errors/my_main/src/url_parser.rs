
use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_query_params(url: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if let Some(query_start) = url.find('?') {
            let query_string = &url[query_start + 1..];
            
            for pair in query_string.split('&') {
                let mut parts = pair.split('=');
                if let Some(key) = parts.next() {
                    let value = parts.next().unwrap_or("");
                    params.insert(key.to_string(), value.to_string());
                }
            }
        }
        
        params
    }
    
    pub fn extract_domain(url: &str) -> Option<String> {
        let url = url.trim();
        
        if url.starts_with("http://") || url.starts_with("https://") {
            if let Some(start) = url.find("://") {
                let after_protocol = &url[start + 3..];
                if let Some(end) = after_protocol.find('/') {
                    return Some(after_protocol[..end].to_string());
                }
                return Some(after_protocol.to_string());
            }
        }
        
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_query_params() {
        let url = "https://example.com/search?q=rust&page=2&sort=desc";
        let params = UrlParser::parse_query_params(url);
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("page"), Some(&"2".to_string()));
        assert_eq!(params.get("sort"), Some(&"desc".to_string()));
        assert_eq!(params.len(), 3);
    }
    
    #[test]
    fn test_extract_domain() {
        assert_eq!(
            UrlParser::extract_domain("https://www.example.com/path"),
            Some("www.example.com".to_string())
        );
        
        assert_eq!(
            UrlParser::extract_domain("http://localhost:8080"),
            Some("localhost:8080".to_string())
        );
        
        assert_eq!(UrlParser::extract_domain("invalid-url"), None);
    }
    
    #[test]
    fn test_empty_query() {
        let url = "https://example.com";
        let params = UrlParser::parse_query_params(url);
        assert!(params.is_empty());
    }
}