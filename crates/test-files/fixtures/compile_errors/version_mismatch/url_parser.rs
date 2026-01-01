use regex::Regex;
use std::collections::HashMap;

pub struct UrlParser {
    url: String,
}

impl UrlParser {
    pub fn new(url: &str) -> Self {
        UrlParser {
            url: url.to_string(),
        }
    }

    pub fn extract_domain(&self) -> Option<String> {
        let re = Regex::new(r"https?://([^/]+)").unwrap();
        re.captures(&self.url)
            .map(|caps| caps[1].to_string())
    }

    pub fn parse_query_params(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        let query_start = self.url.find('?');
        
        if let Some(start) = query_start {
            let query_str = &self.url[start + 1..];
            for pair in query_str.split('&') {
                let parts: Vec<&str> = pair.split('=').collect();
                if parts.len() == 2 {
                    params.insert(parts[0].to_string(), parts[1].to_string());
                }
            }
        }
        
        params
    }

    pub fn is_secure(&self) -> bool {
        self.url.starts_with("https://")
    }

    pub fn get_path(&self) -> Option<String> {
        let re = Regex::new(r"https?://[^/]+(/[^?]*)").unwrap();
        re.captures(&self.url)
            .map(|caps| caps[1].to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_extraction() {
        let parser = UrlParser::new("https://example.com/path?query=value");
        assert_eq!(parser.extract_domain(), Some("example.com".to_string()));
    }

    #[test]
    fn test_query_parsing() {
        let parser = UrlParser::new("https://example.com?name=john&age=30");
        let params = parser.parse_query_params();
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
    }

    #[test]
    fn test_secure_check() {
        let secure_parser = UrlParser::new("https://secure.com");
        let insecure_parser = UrlParser::new("http://insecure.com");
        assert!(secure_parser.is_secure());
        assert!(!insecure_parser.is_secure());
    }
}
use std::collections::HashMap;

pub struct QueryParser;

impl QueryParser {
    pub fn parse(query: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if query.is_empty() {
            return params;
        }
        
        for pair in query.split('&') {
            let mut parts = pair.splitn(2, '=');
            if let Some(key) = parts.next() {
                if let Some(value) = parts.next() {
                    params.insert(key.to_string(), value.to_string());
                } else {
                    params.insert(key.to_string(), String::new());
                }
            }
        }
        
        params
    }
    
    pub fn parse_url(url: &str) -> Option<HashMap<String, String>> {
        if let Some(pos) = url.find('?') {
            let query_string = &url[pos + 1..];
            Some(Self::parse(query_string))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_simple_query() {
        let query = "name=john&age=30&city=newyork";
        let params = QueryParser::parse(query);
        
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
        assert_eq!(params.get("city"), Some(&"newyork".to_string()));
    }
    
    #[test]
    fn test_parse_empty_value() {
        let query = "flag=&empty";
        let params = QueryParser::parse(query);
        
        assert_eq!(params.get("flag"), Some(&"".to_string()));
        assert_eq!(params.get("empty"), Some(&"".to_string()));
    }
    
    #[test]
    fn test_parse_url() {
        let url = "https://example.com/search?q=rust&sort=desc";
        let params = QueryParser::parse_url(url).unwrap();
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("sort"), Some(&"desc".to_string()));
    }
    
    #[test]
    fn test_parse_empty_query() {
        let params = QueryParser::parse("");
        assert!(params.is_empty());
    }
}