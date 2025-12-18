
use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_query_string(url: &str) -> Option<HashMap<String, String>> {
        let query_start = url.find('?')?;
        let query_str = &url[query_start + 1..];
        
        if query_str.is_empty() {
            return Some(HashMap::new());
        }
        
        let mut params = HashMap::new();
        
        for pair in query_str.split('&') {
            let mut parts = pair.splitn(2, '=');
            if let Some(key) = parts.next() {
                let value = parts.next().unwrap_or("");
                params.insert(key.to_string(), value.to_string());
            }
        }
        
        Some(params)
    }
    
    pub fn extract_domain(url: &str) -> Option<String> {
        let after_protocol = if let Some(pos) = url.find("://") {
            &url[pos + 3..]
        } else {
            url
        };
        
        let domain_end = after_protocol.find('/').unwrap_or(after_protocol.len());
        let domain = &after_protocol[..domain_end];
        
        if domain.is_empty() {
            None
        } else {
            Some(domain.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_query_string() {
        let url = "https://example.com/search?q=rust&lang=en&sort=desc";
        let params = UrlParser::parse_query_string(url).unwrap();
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("lang"), Some(&"en".to_string()));
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
            UrlParser::extract_domain("example.com/page"),
            Some("example.com".to_string())
        );
        
        assert_eq!(
            UrlParser::extract_domain("invalid://"),
            None
        );
    }
}