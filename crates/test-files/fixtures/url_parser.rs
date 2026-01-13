use regex::Regex;
use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_domain(url: &str) -> Option<String> {
        let re = Regex::new(r"^(?:https?://)?([^/?#]+)").unwrap();
        re.captures(url)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
    }

    pub fn parse_query_params(url: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        let query_start = url.find('?');
        if let Some(start) = query_start {
            let query_str = &url[start + 1..];
            for pair in query_str.split('&') {
                let mut kv = pair.split('=');
                if let (Some(key), Some(value)) = (kv.next(), kv.next()) {
                    params.insert(key.to_string(), value.to_string());
                }
            }
        }
        params
    }

    pub fn is_valid_url(url: &str) -> bool {
        let re = Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap();
        re.is_match(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_domain() {
        let url = "https://www.example.com/path?query=123";
        assert_eq!(UrlParser::parse_domain(url), Some("www.example.com".to_string()));
        
        let url_no_protocol = "example.com/resource";
        assert_eq!(UrlParser::parse_domain(url_no_protocol), Some("example.com".to_string()));
    }

    #[test]
    fn test_parse_query_params() {
        let url = "https://api.service.com/endpoint?api_key=secret&page=2&sort=desc";
        let params = UrlParser::parse_query_params(url);
        
        assert_eq!(params.get("api_key"), Some(&"secret".to_string()));
        assert_eq!(params.get("page"), Some(&"2".to_string()));
        assert_eq!(params.get("sort"), Some(&"desc".to_string()));
        assert_eq!(params.get("missing"), None);
    }

    #[test]
    fn test_is_valid_url() {
        assert!(UrlParser::is_valid_url("http://valid.com"));
        assert!(UrlParser::is_valid_url("https://sub.domain.co.uk/path"));
        assert!(!UrlParser::is_valid_url("invalid-url"));
        assert!(!UrlParser::is_valid_url("ftp://protocol.com"));
    }
}