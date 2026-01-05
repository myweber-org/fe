use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct ParsedUrl {
    pub protocol: String,
    pub domain: String,
    pub path: String,
    pub query_params: HashMap<String, String>,
    pub fragment: Option<String>,
}

impl ParsedUrl {
    pub fn new(url: &str) -> Result<Self, String> {
        let mut protocol = String::new();
        let mut domain = String::new();
        let mut path = String::new();
        let mut query_params = HashMap::new();
        let mut fragment = None;

        let mut remaining = url;

        if let Some(proto_end) = remaining.find("://") {
            protocol = remaining[..proto_end].to_string();
            remaining = &remaining[proto_end + 3..];
        }

        let domain_end = remaining.find('/').unwrap_or(remaining.len());
        domain = remaining[..domain_end].to_string();
        remaining = &remaining[domain_end..];

        if let Some(fragment_start) = remaining.find('#') {
            fragment = Some(remaining[fragment_start + 1..].to_string());
            remaining = &remaining[..fragment_start];
        }

        if let Some(query_start) = remaining.find('?') {
            path = remaining[..query_start].to_string();
            let query_str = &remaining[query_start + 1..];
            
            for pair in query_str.split('&') {
                if let Some(eq_pos) = pair.find('=') {
                    let key = &pair[..eq_pos];
                    let value = &pair[eq_pos + 1..];
                    query_params.insert(key.to_string(), value.to_string());
                }
            }
        } else {
            path = remaining.to_string();
        }

        if domain.is_empty() {
            return Err("Domain cannot be empty".to_string());
        }

        Ok(ParsedUrl {
            protocol,
            domain,
            path,
            query_params,
            fragment,
        })
    }

    pub fn get_query_param(&self, key: &str) -> Option<&String> {
        self.query_params.get(key)
    }

    pub fn has_fragment(&self) -> bool {
        self.fragment.is_some()
    }
}

pub fn extract_domain(url: &str) -> Option<String> {
    ParsedUrl::new(url).ok().map(|parsed| parsed.domain)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full_url() {
        let url = "https://example.com/path/to/resource?param1=value1&param2=value2#section";
        let parsed = ParsedUrl::new(url).unwrap();
        
        assert_eq!(parsed.protocol, "https");
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.path, "/path/to/resource");
        assert_eq!(parsed.get_query_param("param1"), Some(&"value1".to_string()));
        assert_eq!(parsed.get_query_param("param2"), Some(&"value2".to_string()));
        assert_eq!(parsed.fragment, Some("section".to_string()));
    }

    #[test]
    fn test_parse_url_without_protocol() {
        let url = "example.com/path";
        let parsed = ParsedUrl::new(url).unwrap();
        
        assert_eq!(parsed.protocol, "");
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.path, "/path");
    }

    #[test]
    fn test_extract_domain() {
        assert_eq!(extract_domain("https://rust-lang.org/docs"), Some("rust-lang.org".to_string()));
        assert_eq!(extract_domain("invalid-url"), None);
    }

    #[test]
    fn test_empty_domain() {
        let result = ParsedUrl::new("https:///path");
        assert!(result.is_err());
    }
}