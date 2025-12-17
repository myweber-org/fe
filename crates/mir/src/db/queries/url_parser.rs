
use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_query_string(query: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if query.is_empty() {
            return params;
        }
        
        for pair in query.split('&') {
            let parts: Vec<&str> = pair.split('=').collect();
            if parts.len() == 2 {
                let key = parts[0].to_string();
                let value = parts[1].to_string();
                params.insert(key, value);
            }
        }
        
        params
    }
    
    pub fn extract_domain(url: &str) -> Option<String> {
        let url_lower = url.to_lowercase();
        
        if url_lower.starts_with("http://") || url_lower.starts_with("https://") {
            let without_protocol = if url_lower.starts_with("http://") {
                &url[7..]
            } else {
                &url[8..]
            };
            
            let domain_end = without_protocol.find('/').unwrap_or(without_protocol.len());
            Some(without_protocol[..domain_end].to_string())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_query_string() {
        let query = "name=john&age=30&city=newyork";
        let params = UrlParser::parse_query_string(query);
        
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
        assert_eq!(params.get("city"), Some(&"newyork".to_string()));
        assert_eq!(params.len(), 3);
    }
    
    #[test]
    fn test_empty_query_string() {
        let params = UrlParser::parse_query_string("");
        assert!(params.is_empty());
    }
    
    #[test]
    fn test_extract_domain() {
        let url = "https://www.example.com/path/to/resource";
        let domain = UrlParser::extract_domain(url);
        assert_eq!(domain, Some("www.example.com".to_string()));
    }
    
    #[test]
    fn test_extract_domain_no_protocol() {
        let url = "www.example.com/path";
        let domain = UrlParser::extract_domain(url);
        assert_eq!(domain, None);
    }
}