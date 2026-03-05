
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct ParsedUrl {
    pub scheme: String,
    pub host: String,
    pub path: String,
    pub query_params: HashMap<String, String>,
    pub port: Option<u16>,
}

#[derive(Debug)]
pub enum ParseError {
    InvalidFormat,
    MissingScheme,
    MissingHost,
}

pub fn parse_url(url_str: &str) -> Result<ParsedUrl, ParseError> {
    let parts: Vec<&str> = url_str.split("://").collect();
    if parts.len() != 2 {
        return Err(ParseError::InvalidFormat);
    }

    let scheme = parts[0].to_string();
    if scheme.is_empty() {
        return Err(ParseError::MissingScheme);
    }

    let remaining = parts[1];
    let host_path: Vec<&str> = remaining.splitn(2, '/').collect();
    let host_port = host_path[0];
    let path = if host_path.len() > 1 {
        format!("/{}", host_path[1])
    } else {
        "/".to_string()
    };

    let host_parts: Vec<&str> = host_port.splitn(2, ':').collect();
    let host = host_parts[0].to_string();
    if host.is_empty() {
        return Err(ParseError::MissingHost);
    }

    let port = if host_parts.len() > 1 {
        host_parts[1].parse().ok()
    } else {
        None
    };

    let query_params = extract_query_params(&path);

    Ok(ParsedUrl {
        scheme,
        host,
        path: path.split('?').next().unwrap_or("/").to_string(),
        query_params,
        port,
    })
}

fn extract_query_params(path: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();
    
    if let Some(query_start) = path.find('?') {
        let query_str = &path[query_start + 1..];
        
        for pair in query_str.split('&') {
            let kv: Vec<&str> = pair.splitn(2, '=').collect();
            if kv.len() == 2 && !kv[0].is_empty() {
                params.insert(kv[0].to_string(), kv[1].to_string());
            }
        }
    }
    
    params
}

pub fn is_valid_url(url: &str) -> bool {
    parse_url(url).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_url() {
        let url = "https://example.com/path/to/resource";
        let parsed = parse_url(url).unwrap();
        
        assert_eq!(parsed.scheme, "https");
        assert_eq!(parsed.host, "example.com");
        assert_eq!(parsed.path, "/path/to/resource");
        assert_eq!(parsed.port, None);
        assert!(parsed.query_params.is_empty());
    }

    #[test]
    fn test_parse_url_with_port() {
        let url = "http://localhost:8080/api/data";
        let parsed = parse_url(url).unwrap();
        
        assert_eq!(parsed.scheme, "http");
        assert_eq!(parsed.host, "localhost");
        assert_eq!(parsed.port, Some(8080));
        assert_eq!(parsed.path, "/api/data");
    }

    #[test]
    fn test_parse_url_with_query() {
        let url = "https://api.example.com/search?q=rust&limit=10&sort=desc";
        let parsed = parse_url(url).unwrap();
        
        assert_eq!(parsed.scheme, "https");
        assert_eq!(parsed.host, "api.example.com");
        assert_eq!(parsed.path, "/search");
        assert_eq!(parsed.query_params.get("q"), Some(&"rust".to_string()));
        assert_eq!(parsed.query_params.get("limit"), Some(&"10".to_string()));
        assert_eq!(parsed.query_params.get("sort"), Some(&"desc".to_string()));
    }

    #[test]
    fn test_invalid_urls() {
        assert!(parse_url("").is_err());
        assert!(parse_url("example.com").is_err());
        assert!(parse_url("://example.com").is_err());
        assert!(parse_url("http://").is_err());
    }

    #[test]
    fn test_is_valid_url() {
        assert!(is_valid_url("https://example.com"));
        assert!(is_valid_url("ftp://files.server:21/download"));
        assert!(!is_valid_url("invalid-url"));
        assert!(!is_valid_url("http://"));
    }
}