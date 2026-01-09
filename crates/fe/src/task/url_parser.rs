
use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_domain(url: &str) -> Option<String> {
        let url = url.trim();
        if url.is_empty() {
            return None;
        }

        let url_lower = url.to_lowercase();
        let prefixes = ["http://", "https://", "ftp://", "//"];
        
        let mut clean_url = url;
        for prefix in prefixes.iter() {
            if url_lower.starts_with(prefix) {
                clean_url = &url[prefix.len()..];
                break;
            }
        }

        let domain_end = clean_url.find('/').unwrap_or(clean_url.len());
        let domain_part = &clean_url[..domain_end];
        
        if domain_part.is_empty() {
            None
        } else {
            Some(domain_part.to_string())
        }
    }

    pub fn parse_query_params(url: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if let Some(query_start) = url.find('?') {
            let query_string = &url[query_start + 1..];
            
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

    pub fn extract_path(url: &str) -> Option<String> {
        let url = url.trim();
        if url.is_empty() {
            return None;
        }

        let url_lower = url.to_lowercase();
        let prefixes = ["http://", "https://", "ftp://", "//"];
        
        let mut clean_url = url;
        for prefix in prefixes.iter() {
            if url_lower.starts_with(prefix) {
                clean_url = &url[prefix.len()..];
                break;
            }
        }

        if let Some(path_start) = clean_url.find('/') {
            let path_with_query = &clean_url[path_start..];
            if let Some(query_start) = path_with_query.find('?') {
                Some(path_with_query[..query_start].to_string())
            } else {
                Some(path_with_query.to_string())
            }
        } else {
            Some("/".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_domain() {
        assert_eq!(UrlParser::parse_domain("https://example.com/path"), Some("example.com".to_string()));
        assert_eq!(UrlParser::parse_domain("http://sub.example.co.uk:8080"), Some("sub.example.co.uk:8080".to_string()));
        assert_eq!(UrlParser::parse_domain("ftp://files.server.com"), Some("files.server.com".to_string()));
        assert_eq!(UrlParser::parse_domain("//cdn.example.com/resource"), Some("cdn.example.com".to_string()));
        assert_eq!(UrlParser::parse_domain("invalid-url"), Some("invalid-url".to_string()));
        assert_eq!(UrlParser::parse_domain(""), None);
    }

    #[test]
    fn test_parse_query_params() {
        let params = UrlParser::parse_query_params("https://example.com/search?q=rust&lang=en&sort=desc");
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("lang"), Some(&"en".to_string()));
        assert_eq!(params.get("sort"), Some(&"desc".to_string()));
        
        let empty_params = UrlParser::parse_query_params("https://example.com");
        assert!(empty_params.is_empty());
    }

    #[test]
    fn test_extract_path() {
        assert_eq!(UrlParser::extract_path("https://example.com/api/users"), Some("/api/users".to_string()));
        assert_eq!(UrlParser::extract_path("https://example.com/api/users?id=123"), Some("/api/users".to_string()));
        assert_eq!(UrlParser::extract_path("https://example.com"), Some("/".to_string()));
        assert_eq!(UrlParser::extract_path("//example.com"), Some("/".to_string()));
        assert_eq!(UrlParser::extract_path(""), None);
    }
}