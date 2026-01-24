use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_query_string(query: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if query.is_empty() {
            return params;
        }

        for pair in query.split('&') {
            let mut parts = pair.splitn(2, '=');
            if let Some(key) = parts.next() {
                let value = parts.next().unwrap_or("");
                params.insert(key.to_string(), value.to_string());
            }
        }
        
        params
    }

    pub fn extract_domain(url: &str) -> Option<String> {
        let url_lower = url.to_lowercase();
        
        if url_lower.starts_with("http://") || url_lower.starts_with("https://") {
            let without_scheme = &url[url.find("://").unwrap() + 3..];
            let domain_end = without_scheme.find('/').unwrap_or(without_scheme.len());
            Some(without_scheme[..domain_end].to_string())
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
        let query = "name=john&age=30&city=new+york";
        let params = UrlParser::parse_query_string(query);
        
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
        assert_eq!(params.get("city"), Some(&"new+york".to_string()));
    }

    #[test]
    fn test_extract_domain() {
        assert_eq!(
            UrlParser::extract_domain("https://example.com/path"),
            Some("example.com".to_string())
        );
        assert_eq!(
            UrlParser::extract_domain("http://sub.domain.co.uk/"),
            Some("sub.domain.co.uk".to_string())
        );
        assert_eq!(UrlParser::extract_domain("invalid-url"), None);
    }
}use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_query_string(url: &str) -> HashMap<String, String> {
        let mut query_params = HashMap::new();
        
        if let Some(query_start) = url.find('?') {
            let query_str = &url[query_start + 1..];
            
            for param_pair in query_str.split('&') {
                let parts: Vec<&str> = param_pair.split('=').collect();
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
        
        if url_lower.starts_with("http://") {
            let without_protocol = &url[7..];
            return without_protocol.split('/').next().map(|s| s.to_string());
        }
        
        if url_lower.starts_with("https://") {
            let without_protocol = &url[8..];
            return without_protocol.split('/').next().map(|s| s.to_string());
        }
        
        url.split('/').next().map(|s| s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_query_string() {
        let url = "https://example.com/search?q=rust&page=2&sort=recent";
        let params = UrlParser::parse_query_string(url);
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("page"), Some(&"2".to_string()));
        assert_eq!(params.get("sort"), Some(&"recent".to_string()));
        assert_eq!(params.get("nonexistent"), None);
    }
    
    #[test]
    fn test_extract_domain() {
        assert_eq!(
            UrlParser::extract_domain("https://www.example.com/path"),
            Some("www.example.com".to_string())
        );
        
        assert_eq!(
            UrlParser::extract_domain("http://localhost:8080/api"),
            Some("localhost:8080".to_string())
        );
        
        assert_eq!(
            UrlParser::extract_domain("example.com/page"),
            Some("example.com".to_string())
        );
    }
}