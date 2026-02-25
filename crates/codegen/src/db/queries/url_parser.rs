use std::collections::HashMap;

pub struct QueryParser;

impl QueryParser {
    pub fn parse(query_string: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if query_string.is_empty() {
            return params;
        }

        for pair in query_string.split('&') {
            let mut parts = pair.splitn(2, '=');
            if let Some(key) = parts.next() {
                let value = parts.next().unwrap_or("");
                params.insert(
                    key.to_string(),
                    urlencoding::decode(value)
                        .unwrap_or_else(|_| value.into())
                        .to_string(),
                );
            }
        }
        
        params
    }

    pub fn parse_from_url(url: &str) -> Option<HashMap<String, String>> {
        url.split('?')
            .nth(1)
            .map(|query| Self::parse(query))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_query() {
        let query = "name=john&age=25";
        let params = QueryParser::parse(query);
        
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"25".to_string()));
    }

    #[test]
    fn test_parse_encoded_values() {
        let query = "city=New%20York&country=USA";
        let params = QueryParser::parse(query);
        
        assert_eq!(params.get("city"), Some(&"New York".to_string()));
        assert_eq!(params.get("country"), Some(&"USA".to_string()));
    }

    #[test]
    fn test_parse_empty_query() {
        let params = QueryParser::parse("");
        assert!(params.is_empty());
    }

    #[test]
    fn test_parse_from_url() {
        let url = "https://example.com/search?q=rust&lang=en";
        let params = QueryParser::parse_from_url(url).unwrap();
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("lang"), Some(&"en".to_string()));
    }
}use std::collections::HashMap;
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

        let parts: Vec<&str> = url.split("://").collect();
        if parts.len() == 2 {
            protocol = parts[0].to_string();
            let rest = parts[1];
            
            let domain_end = rest.find('/').unwrap_or(rest.len());
            domain = rest[..domain_end].to_string();
            
            let path_start = domain_end;
            if path_start < rest.len() {
                let path_and_more = &rest[path_start..];
                
                let fragment_split: Vec<&str> = path_and_more.split('#').collect();
                let path_with_query = fragment_split[0];
                if fragment_split.len() > 1 {
                    fragment = Some(fragment_split[1].to_string());
                }
                
                let query_split: Vec<&str> = path_with_query.split('?').collect();
                path = query_split[0].to_string();
                
                if query_split.len() > 1 {
                    for param in query_split[1].split('&') {
                        let key_value: Vec<&str> = param.split('=').collect();
                        if key_value.len() == 2 {
                            query_params.insert(
                                key_value[0].to_string(),
                                key_value[1].to_string()
                            );
                        }
                    }
                }
            }
        } else {
            return Err("Invalid URL format".into());
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
            let last_two = parts[parts.len()-2..].join(".");
            Some(last_two)
        } else {
            None
        }
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
    fn test_parse_basic_url() {
        let url = "https://example.com/path/to/resource";
        let parsed = ParsedUrl::new(url).unwrap();
        
        assert_eq!(parsed.protocol, "https");
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.path, "/path/to/resource");
        assert!(parsed.query_params.is_empty());
        assert_eq!(parsed.fragment, None);
    }

    #[test]
    fn test_parse_url_with_query_and_fragment() {
        let url = "https://api.example.com/search?q=rust&page=2#results";
        let parsed = ParsedUrl::new(url).unwrap();
        
        assert_eq!(parsed.protocol, "https");
        assert_eq!(parsed.domain, "api.example.com");
        assert_eq!(parsed.path, "/search");
        assert_eq!(parsed.get_query_param("q"), Some(&"rust".to_string()));
        assert_eq!(parsed.get_query_param("page"), Some(&"2".to_string()));
        assert_eq!(parsed.fragment, Some("results".to_string()));
    }

    #[test]
    fn test_get_root_domain() {
        let url = "https://subdomain.example.co.uk/path";
        let parsed = ParsedUrl::new(url).unwrap();
        
        assert_eq!(parsed.get_root_domain(), Some("co.uk".to_string()));
    }

    #[test]
    fn test_invalid_url() {
        let url = "not-a-valid-url";
        let result = ParsedUrl::new(url);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_valid_url() {
        assert!(is_valid_url("https://example.com"));
        assert!(!is_valid_url("invalid-url"));
    }
}