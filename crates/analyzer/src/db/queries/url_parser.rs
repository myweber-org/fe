
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct ParsedUrl {
    pub protocol: String,
    pub domain: String,
    pub path: String,
    pub query_params: HashMap<String, String>,
    pub port: Option<u16>,
}

impl ParsedUrl {
    pub fn new(url: &str) -> Result<Self, String> {
        if url.trim().is_empty() {
            return Err("URL cannot be empty".to_string());
        }

        let mut protocol = "https".to_string();
        let mut remaining = url;

        if let Some(proto_end) = url.find("://") {
            protocol = url[..proto_end].to_string().to_lowercase();
            remaining = &url[proto_end + 3..];
        }

        let mut domain_port = remaining;
        let mut path = "/".to_string();
        let mut query_params = HashMap::new();

        if let Some(path_start) = remaining.find('/') {
            domain_port = &remaining[..path_start];
            let path_and_query = &remaining[path_start..];

            if let Some(query_start) = path_and_query.find('?') {
                path = path_and_query[..query_start].to_string();
                let query_str = &path_and_query[query_start + 1..];

                for pair in query_str.split('&') {
                    if pair.is_empty() {
                        continue;
                    }
                    let mut key_value = pair.splitn(2, '=');
                    if let Some(key) = key_value.next() {
                        let value = key_value.next().unwrap_or("").to_string();
                        query_params.insert(key.to_string(), value);
                    }
                }
            } else {
                path = path_and_query.to_string();
            }
        }

        let mut domain = domain_port.to_string();
        let mut port = None;

        if let Some(port_start) = domain_port.find(':') {
            domain = domain_port[..port_start].to_string();
            if let Ok(parsed_port) = domain_port[port_start + 1..].parse::<u16>() {
                port = Some(parsed_port);
            }
        }

        if domain.is_empty() {
            return Err("Domain cannot be empty".to_string());
        }

        Ok(ParsedUrl {
            protocol,
            domain,
            path,
            query_params,
            port,
        })
    }

    pub fn get_query_param(&self, key: &str) -> Option<&String> {
        self.query_params.get(key)
    }

    pub fn has_query_params(&self) -> bool {
        !self.query_params.is_empty()
    }

    pub fn full_url(&self) -> String {
        let mut url = format!("{}://{}", self.protocol, self.domain);
        
        if let Some(port) = self.port {
            url.push_str(&format!(":{}", port));
        }
        
        url.push_str(&self.path);
        
        if self.has_query_params() {
            url.push('?');
            let mut first = true;
            for (key, value) in &self.query_params {
                if !first {
                    url.push('&');
                }
                url.push_str(&format!("{}={}", key, value));
                first = false;
            }
        }
        
        url
    }
}

pub fn is_valid_url(url: &str) -> bool {
    ParsedUrl::new(url).is_ok()
}

pub fn extract_domain(url: &str) -> Option<String> {
    ParsedUrl::new(url).ok().map(|parsed| parsed.domain)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_url_parsing() {
        let url = "https://example.com/path/to/resource";
        let parsed = ParsedUrl::new(url).unwrap();
        
        assert_eq!(parsed.protocol, "https");
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.path, "/path/to/resource");
        assert!(!parsed.has_query_params());
        assert_eq!(parsed.port, None);
    }

    #[test]
    fn test_url_with_query_params() {
        let url = "http://api.example.com/search?q=rust&limit=10&sort=desc";
        let parsed = ParsedUrl::new(url).unwrap();
        
        assert_eq!(parsed.protocol, "http");
        assert_eq!(parsed.domain, "api.example.com");
        assert_eq!(parsed.path, "/search");
        assert_eq!(parsed.get_query_param("q"), Some(&"rust".to_string()));
        assert_eq!(parsed.get_query_param("limit"), Some(&"10".to_string()));
        assert_eq!(parsed.get_query_param("sort"), Some(&"desc".to_string()));
        assert_eq!(parsed.get_query_param("nonexistent"), None);
    }

    #[test]
    fn test_url_with_port() {
        let url = "https://localhost:8080/api/v1/users";
        let parsed = ParsedUrl::new(url).unwrap();
        
        assert_eq!(parsed.protocol, "https");
        assert_eq!(parsed.domain, "localhost");
        assert_eq!(parsed.port, Some(8080));
        assert_eq!(parsed.path, "/api/v1/users");
    }

    #[test]
    fn test_empty_url() {
        let result = ParsedUrl::new("");
        assert!(result.is_err());
    }

    #[test]
    fn test_url_without_protocol() {
        let url = "example.com/path";
        let parsed = ParsedUrl::new(url).unwrap();
        
        assert_eq!(parsed.protocol, "https");
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.path, "/path");
    }

    #[test]
    fn test_full_url_reconstruction() {
        let original = "https://api.example.com:3000/search?query=test&page=2";
        let parsed = ParsedUrl::new(original).unwrap();
        let reconstructed = parsed.full_url();
        
        assert_eq!(reconstructed, original);
    }

    #[test]
    fn test_is_valid_url() {
        assert!(is_valid_url("https://example.com"));
        assert!(is_valid_url("http://localhost:3000"));
        assert!(is_valid_url("example.com/path?query=test"));
        assert!(!is_valid_url(""));
    }

    #[test]
    fn test_extract_domain() {
        assert_eq!(extract_domain("https://example.com/path"), Some("example.com".to_string()));
        assert_eq!(extract_domain("sub.domain.co.uk/api"), Some("sub.domain.co.uk".to_string()));
        assert_eq!(extract_domain(""), None);
    }
}use std::collections::HashMap;

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
            let without_scheme = if url_lower.starts_with("http://") {
                &url[7..]
            } else {
                &url[8..]
            };
            
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
            UrlParser::extract_domain("http://sub.domain.co.uk"),
            Some("sub.domain.co.uk".to_string())
        );
        
        assert_eq!(UrlParser::extract_domain("invalid-url"), None);
    }
}