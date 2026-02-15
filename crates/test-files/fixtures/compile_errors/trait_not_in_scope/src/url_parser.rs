use regex::Regex;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct ParsedUrl {
    pub scheme: String,
    pub host: String,
    pub path: String,
    pub query_params: HashMap<String, String>,
    pub is_valid: bool,
}

pub fn parse_url(url: &str) -> ParsedUrl {
    let url_regex = Regex::new(r"^(?P<scheme>https?|ftp)://(?P<host>[^/]+)(?P<path>/[^?]*)?(?:\?(?P<query>.*))?$").unwrap();
    
    let mut parsed = ParsedUrl {
        scheme: String::new(),
        host: String::new(),
        path: String::from("/"),
        query_params: HashMap::new(),
        is_valid: false,
    };

    if let Some(captures) = url_regex.captures(url) {
        parsed.is_valid = true;
        
        if let Some(scheme) = captures.name("scheme") {
            parsed.scheme = scheme.as_str().to_string();
        }
        
        if let Some(host) = captures.name("host") {
            parsed.host = host.as_str().to_string();
        }
        
        if let Some(path) = captures.name("path") {
            parsed.path = path.as_str().to_string();
        }
        
        if let Some(query) = captures.name("query") {
            parse_query_string(query.as_str(), &mut parsed.query_params);
        }
    }
    
    parsed
}

fn parse_query_string(query: &str, params: &mut HashMap<String, String>) {
    for pair in query.split('&') {
        let mut parts = pair.splitn(2, '=');
        if let Some(key) = parts.next() {
            let value = parts.next().unwrap_or("").to_string();
            params.insert(key.to_string(), value);
        }
    }
}

pub fn validate_url(url: &str) -> bool {
    parse_url(url).is_valid
}

pub fn extract_domain(url: &str) -> Option<String> {
    let parsed = parse_url(url);
    if parsed.is_valid {
        Some(parsed.host)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_http_url() {
        let url = "http://example.com/path?key=value&name=test";
        let parsed = parse_url(url);
        
        assert!(parsed.is_valid);
        assert_eq!(parsed.scheme, "http");
        assert_eq!(parsed.host, "example.com");
        assert_eq!(parsed.path, "/path");
        assert_eq!(parsed.query_params.get("key"), Some(&"value".to_string()));
        assert_eq!(parsed.query_params.get("name"), Some(&"test".to_string()));
    }

    #[test]
    fn test_valid_https_url_no_query() {
        let url = "https://api.github.com/users";
        let parsed = parse_url(url);
        
        assert!(parsed.is_valid);
        assert_eq!(parsed.scheme, "https");
        assert_eq!(parsed.host, "api.github.com");
        assert_eq!(parsed.path, "/users");
        assert!(parsed.query_params.is_empty());
    }

    #[test]
    fn test_invalid_url() {
        let url = "not-a-valid-url";
        let parsed = parse_url(url);
        assert!(!parsed.is_valid);
    }

    #[test]
    fn test_extract_domain() {
        assert_eq!(extract_domain("https://www.rust-lang.org/learn"), Some("www.rust-lang.org".to_string()));
        assert_eq!(extract_domain("invalid-url"), None);
    }

    #[test]
    fn test_validate_url() {
        assert!(validate_url("ftp://files.example.com/data.txt"));
        assert!(!validate_url("://missing-scheme"));
    }
}