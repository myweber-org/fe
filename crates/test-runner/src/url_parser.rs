use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct ParsedUrl {
    pub protocol: String,
    pub domain: String,
    pub path: String,
    pub query_params: HashMap<String, String>,
}

impl ParsedUrl {
    pub fn parse(url: &str) -> Result<Self, &'static str> {
        let mut protocol = String::new();
        let mut domain = String::new();
        let mut path = String::new();
        let mut query_params = HashMap::new();

        let parts: Vec<&str> = url.split("://").collect();
        if parts.len() != 2 {
            return Err("Invalid URL format");
        }

        protocol = parts[0].to_string();
        let rest = parts[1];

        let domain_end = rest.find('/').unwrap_or(rest.len());
        domain = rest[..domain_end].to_string();

        if domain_end < rest.len() {
            let path_with_query = &rest[domain_end..];
            let path_parts: Vec<&str> = path_with_query.split('?').collect();
            path = path_parts[0].to_string();

            if path_parts.len() > 1 {
                let query_string = path_parts[1];
                for pair in query_string.split('&') {
                    let kv: Vec<&str> = pair.split('=').collect();
                    if kv.len() == 2 {
                        query_params.insert(kv[0].to_string(), kv[1].to_string());
                    }
                }
            }
        }

        Ok(ParsedUrl {
            protocol,
            domain,
            path,
            query_params,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_url() {
        let url = "https://example.com/path/to/resource";
        let parsed = ParsedUrl::parse(url).unwrap();
        assert_eq!(parsed.protocol, "https");
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.path, "/path/to/resource");
        assert!(parsed.query_params.is_empty());
    }

    #[test]
    fn test_parse_url_with_query() {
        let url = "http://test.org/api?key=value&page=2";
        let parsed = ParsedUrl::parse(url).unwrap();
        assert_eq!(parsed.protocol, "http");
        assert_eq!(parsed.domain, "test.org");
        assert_eq!(parsed.path, "/api");
        assert_eq!(parsed.query_params.get("key"), Some(&"value".to_string()));
        assert_eq!(parsed.query_params.get("page"), Some(&"2".to_string()));
    }

    #[test]
    fn test_parse_invalid_url() {
        let url = "invalid-url";
        let result = ParsedUrl::parse(url);
        assert!(result.is_err());
    }
}