
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, PartialEq)]
pub struct ParsedUrl {
    pub protocol: String,
    pub domain: String,
    pub path: String,
    pub query_params: HashMap<String, String>,
    pub fragment: Option<String>,
}

impl ParsedUrl {
    pub fn new(url: &str) -> Result<Self, Box<dyn Error>> {
        let mut protocol = String::new();
        let mut domain = String::new();
        let mut path = String::new();
        let mut query_params = HashMap::new();
        let mut fragment = None;

        let url_lower = url.to_lowercase();
        let url_str = url_lower.trim();

        if url_str.is_empty() {
            return Err("Empty URL provided".into());
        }

        let protocol_end = url_str.find("://");
        let remaining = if let Some(pos) = protocol_end {
            protocol = url_str[..pos].to_string();
            &url_str[pos + 3..]
        } else {
            url_str
        };

        let domain_end = remaining.find('/').unwrap_or(remaining.len());
        domain = remaining[..domain_end].to_string();

        if domain.is_empty() {
            return Err("No domain found in URL".into());
        }

        let path_start = domain_end;
        let mut path_and_query = &remaining[path_start..];

        let fragment_pos = path_and_query.find('#');
        if let Some(pos) = fragment_pos {
            fragment = Some(path_and_query[pos + 1..].to_string());
            path_and_query = &path_and_query[..pos];
        }

        let query_pos = path_and_query.find('?');
        if let Some(pos) = query_pos {
            path = path_and_query[..pos].to_string();
            let query_str = &path_and_query[pos + 1..];
            
            for pair in query_str.split('&') {
                if pair.is_empty() {
                    continue;
                }
                let kv: Vec<&str> = pair.splitn(2, '=').collect();
                if kv.len() == 2 {
                    query_params.insert(kv[0].to_string(), kv[1].to_string());
                } else {
                    query_params.insert(kv[0].to_string(), String::new());
                }
            }
        } else {
            path = path_and_query.to_string();
        }

        if path.is_empty() {
            path = "/".to_string();
        }

        Ok(ParsedUrl {
            protocol,
            domain,
            path,
            query_params,
            fragment,
        })
    }

    pub fn get_root_domain(&self) -> Option<String> {
        let parts: Vec<&str> = self.domain.split('.').collect();
        if parts.len() >= 2 {
            let last_two = &parts[parts.len() - 2..];
            Some(last_two.join("."))
        } else {
            None
        }
    }

    pub fn has_query_param(&self, key: &str) -> bool {
        self.query_params.contains_key(key)
    }

    pub fn get_query_param(&self, key: &str) -> Option<&String> {
        self.query_params.get(key)
    }
}

pub fn is_valid_url(url: &str) -> bool {
    ParsedUrl::new(url).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full_url() {
        let url = "https://www.example.com/path/to/page?param1=value1&param2=value2#section";
        let parsed = ParsedUrl::new(url).unwrap();
        
        assert_eq!(parsed.protocol, "https");
        assert_eq!(parsed.domain, "www.example.com");
        assert_eq!(parsed.path, "/path/to/page");
        assert_eq!(parsed.query_params.get("param1"), Some(&"value1".to_string()));
        assert_eq!(parsed.query_params.get("param2"), Some(&"value2".to_string()));
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
    fn test_parse_url_with_empty_query() {
        let url = "https://example.com/page?";
        let parsed = ParsedUrl::new(url).unwrap();
        
        assert_eq!(parsed.path, "/page");
        assert!(parsed.query_params.is_empty());
    }

    #[test]
    fn test_root_domain_extraction() {
        let url = "https://sub.domain.example.co.uk/path";
        let parsed = ParsedUrl::new(url).unwrap();
        
        assert_eq!(parsed.get_root_domain(), Some("co.uk".to_string()));
    }

    #[test]
    fn test_invalid_url() {
        let url = "";
        let result = ParsedUrl::new(url);
        assert!(result.is_err());
    }

    #[test]
    fn test_query_param_access() {
        let url = "https://example.com?search=rust&page=1";
        let parsed = ParsedUrl::new(url).unwrap();
        
        assert!(parsed.has_query_param("search"));
        assert_eq!(parsed.get_query_param("search"), Some(&"rust".to_string()));
        assert_eq!(parsed.get_query_param("nonexistent"), None);
    }

    #[test]
    fn test_is_valid_url() {
        assert!(is_valid_url("https://example.com"));
        assert!(is_valid_url("example.com/path"));
        assert!(!is_valid_url(""));
    }
}