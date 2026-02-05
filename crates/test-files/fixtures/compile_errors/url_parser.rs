use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_domain(url: &str) -> Option<String> {
        let url = url.trim();
        if url.is_empty() {
            return None;
        }

        let after_protocol = if let Some(stripped) = url.strip_prefix("http://") {
            stripped
        } else if let Some(stripped) = url.strip_prefix("https://") {
            stripped
        } else {
            url
        };

        let domain_end = after_protocol.find('/').unwrap_or(after_protocol.len());
        let domain = &after_protocol[..domain_end];

        if domain.is_empty() {
            None
        } else {
            Some(domain.to_string())
        }
    }

    pub fn parse_query_params(url: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if let Some(query_start) = url.find('?') {
            let query_string = &url[query_start + 1..];
            
            for pair in query_string.split('&') {
                let parts: Vec<&str> = pair.split('=').collect();
                if parts.len() == 2 {
                    params.insert(parts[0].to_string(), parts[1].to_string());
                }
            }
        }
        
        params
    }

    pub fn parse(url: &str) -> (Option<String>, HashMap<String, String>) {
        let domain = Self::parse_domain(url);
        let params = Self::parse_query_params(url);
        (domain, params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_domain() {
        assert_eq!(
            UrlParser::parse_domain("https://example.com/path"),
            Some("example.com".to_string())
        );
        assert_eq!(
            UrlParser::parse_domain("http://sub.example.com"),
            Some("sub.example.com".to_string())
        );
        assert_eq!(
            UrlParser::parse_domain("example.com/page"),
            Some("example.com".to_string())
        );
        assert_eq!(UrlParser::parse_domain(""), None);
    }

    #[test]
    fn test_parse_query_params() {
        let params = UrlParser::parse_query_params(
            "https://example.com/search?q=rust&lang=en&page=1"
        );
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("lang"), Some(&"en".to_string()));
        assert_eq!(params.get("page"), Some(&"1".to_string()));
        assert_eq!(params.get("missing"), None);
    }

    #[test]
    fn test_parse_full() {
        let (domain, params) = UrlParser::parse(
            "https://api.example.com/v1/users?id=123&name=john"
        );
        
        assert_eq!(domain, Some("api.example.com".to_string()));
        assert_eq!(params.get("id"), Some(&"123".to_string()));
        assert_eq!(params.get("name"), Some(&"john".to_string()));
    }
}