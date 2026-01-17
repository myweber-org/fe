use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_query_string(url: &str) -> HashMap<String, String> {
        let mut query_params = HashMap::new();
        
        if let Some(query_start) = url.find('?') {
            let query_string = &url[query_start + 1..];
            
            for pair in query_string.split('&') {
                let parts: Vec<&str> = pair.split('=').collect();
                if parts.len() == 2 {
                    let key = parts[0].to_string();
                    let value = parts[1].to_string();
                    query_params.insert(key, value);
                }
            }
        }
        
        query_params
    }
    
    pub fn extract_domain(url: &str) -> Option<String> {
        let url_lower = url.to_lowercase();
        
        if url_lower.starts_with("http://") || url_lower.starts_with("https://") {
            if let Some(domain_start) = url.find("://") {
                let after_protocol = &url[domain_start + 3..];
                if let Some(domain_end) = after_protocol.find('/') {
                    return Some(after_protocol[..domain_end].to_string());
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
    fn test_parse_query_string() {
        let url = "https://example.com/search?q=rust&page=2&sort=desc";
        let params = UrlParser::parse_query_string(url);
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("page"), Some(&"2".to_string()));
        assert_eq!(params.get("sort"), Some(&"desc".to_string()));
        assert_eq!(params.len(), 3);
    }
    
    #[test]
    fn test_extract_domain() {
        let url1 = "https://www.example.com/path/to/resource";
        let url2 = "http://subdomain.example.co.uk";
        let url3 = "invalid-url";
        
        assert_eq!(UrlParser::extract_domain(url1), Some("www.example.com".to_string()));
        assert_eq!(UrlParser::extract_domain(url2), Some("subdomain.example.co.uk".to_string()));
        assert_eq!(UrlParser::extract_domain(url3), None);
    }
    
    #[test]
    fn test_empty_query_string() {
        let url = "https://example.com";
        let params = UrlParser::parse_query_string(url);
        
        assert!(params.is_empty());
    }
}