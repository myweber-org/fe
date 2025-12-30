use regex::Regex;
use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_domain(url: &str) -> Option<String> {
        let re = Regex::new(r"^(?:https?://)?(?:www\.)?([^/?#]+)").unwrap();
        re.captures(url)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }

    pub fn parse_query_params(url: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        let query_start = url.find('?');
        
        if let Some(start) = query_start {
            let query_str = &url[start + 1..];
            for pair in query_str.split('&') {
                let parts: Vec<&str> = pair.split('=').collect();
                if parts.len() == 2 {
                    params.insert(parts[0].to_string(), parts[1].to_string());
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
    fn test_domain_parsing() {
        let test_cases = vec![
            ("https://www.example.com/path", Some("example.com".to_string())),
            ("http://sub.domain.co.uk", Some("sub.domain.co.uk".to_string())),
            ("invalid-url", None),
        ];

        for (input, expected) in test_cases {
            assert_eq!(UrlParser::parse_domain(input), expected);
        }
    }

    #[test]
    fn test_query_parsing() {
        let url = "https://example.com/search?q=rust&sort=desc&page=2";
        let params = UrlParser::parse_query_params(url);
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("sort"), Some(&"desc".to_string()));
        assert_eq!(params.get("page"), Some(&"2".to_string()));
    }

    #[test]
    fn test_url_validation() {
        assert!(UrlParser::is_valid_url("https://example.com"));
        assert!(UrlParser::is_valid_url("http://localhost:8080"));
        assert!(!UrlParser::is_valid_url("not-a-url"));
    }
}