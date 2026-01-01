
use std::collections::HashMap;

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

        let path_start = remaining.find('/').unwrap_or(remaining.len());
        domain = remaining[..path_start].to_string();
        
        if path_start < remaining.len() {
            remaining = &remaining[path_start..];
        } else {
            remaining = "";
        }

        let fragment_split: Vec<&str> = remaining.splitn(2, '#').collect();
        let path_and_query = fragment_split[0];
        
        if fragment_split.len() > 1 {
            fragment = Some(fragment_split[1].to_string());
        }

        let query_split: Vec<&str> = path_and_query.splitn(2, '?').collect();
        path = query_split[0].to_string();
        
        if query_split.len() > 1 {
            for param in query_split[1].split('&') {
                let pair: Vec<&str> = param.splitn(2, '=').collect();
                if pair.len() == 2 {
                    query_params.insert(pair[0].to_string(), pair[1].to_string());
                }
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
        let url = "https://example.com/path/to/resource?key1=value1&key2=value2#section";
        let parsed = ParsedUrl::new(url).unwrap();
        
        assert_eq!(parsed.protocol, "https");
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.path, "/path/to/resource");
        assert_eq!(parsed.get_query_param("key1"), Some(&"value1".to_string()));
        assert_eq!(parsed.get_query_param("key2"), Some(&"value2".to_string()));
        assert_eq!(parsed.fragment, Some("section".to_string()));
    }

    #[test]
    fn test_parse_url_without_protocol() {
        let url = "example.com/path";
        let parsed = ParsedUrl::new(url).unwrap();
        
        assert_eq!(parsed.protocol, "");
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.path, "/path");
        assert!(parsed.query_params.is_empty());
        assert_eq!(parsed.fragment, None);
    }

    #[test]
    fn test_extract_domain_function() {
        let url = "https://subdomain.example.co.uk/path";
        let domain = extract_domain(url).unwrap();
        assert_eq!(domain, "subdomain.example.co.uk");
    }

    #[test]
    fn test_invalid_url() {
        let url = "";
        let result = ParsedUrl::new(url);
        assert!(result.is_err());
    }
}