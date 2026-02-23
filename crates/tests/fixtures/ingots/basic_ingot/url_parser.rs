
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
        let mut rest = url;
        
        if let Some(sep) = url.find("://") {
            protocol = url[..sep].to_string();
            rest = &url[sep + 3..];
        }
        
        let mut domain_end = rest.len();
        let mut path_start = rest.len();
        let mut query_start = rest.len();
        let mut fragment_start = rest.len();
        
        if let Some(pos) = rest.find('/') {
            domain_end = pos;
            path_start = pos;
        }
        
        if let Some(pos) = rest.find('?') {
            if pos < query_start {
                query_start = pos;
                if path_start > query_start {
                    path_start = query_start;
                }
            }
        }
        
        if let Some(pos) = rest.find('#') {
            fragment_start = pos;
        }
        
        let domain = rest[..domain_end].to_string();
        let path = if path_start < fragment_start.min(query_start) {
            rest[path_start..fragment_start.min(query_start)].to_string()
        } else {
            String::new()
        };
        
        let query_params = if query_start < fragment_start {
            let query_str = &rest[query_start + 1..fragment_start];
            parse_query_string(query_str)
        } else {
            HashMap::new()
        };
        
        let fragment = if fragment_start < rest.len() {
            Some(rest[fragment_start + 1..].to_string())
        } else {
            None
        };
        
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

fn parse_query_string(query: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();
    
    for pair in query.split('&') {
        if let Some(sep) = pair.find('=') {
            let key = &pair[..sep];
            let value = &pair[sep + 1..];
            params.insert(key.to_string(), value.to_string());
        }
    }
    
    params
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
    fn test_parse_url_with_only_domain() {
        let url = "https://example.com";
        let parsed = ParsedUrl::new(url).unwrap();
        
        assert_eq!(parsed.protocol, "https");
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.path, "");
        assert!(parsed.query_params.is_empty());
        assert_eq!(parsed.fragment, None);
    }
}