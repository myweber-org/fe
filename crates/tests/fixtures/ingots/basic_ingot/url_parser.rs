use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_query_string(query: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if query.is_empty() {
            return params;
        }

        for pair in query.split('&') {
            let mut parts = pair.splitn(2, '=');
            if let Some(key) = parts.next() {
                let value = parts.next().unwrap_or("");
                params.insert(key.to_string(), value.to_string());
            }
        }
        
        params
    }

    pub fn extract_domain(url: &str) -> Option<String> {
        let url_lower = url.to_lowercase();
        let prefixes = ["http://", "https://", "www."];
        
        let mut domain_start = 0;
        for prefix in &prefixes {
            if url_lower.starts_with(prefix) {
                domain_start = prefix.len();
                break;
            }
        }

        let remaining = &url[domain_start..];
        let domain_end = remaining.find('/').unwrap_or(remaining.len());
        
        if domain_end > 0 {
            Some(remaining[..domain_end].to_string())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_query_string() {
        let query = "name=john&age=30&city=new+york";
        let params = UrlParser::parse_query_string(query);
        
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
        assert_eq!(params.get("city"), Some(&"new+york".to_string()));
        assert_eq!(params.len(), 3);
    }

    #[test]
    fn test_empty_query_string() {
        let params = UrlParser::parse_query_string("");
        assert!(params.is_empty());
    }

    #[test]
    fn test_extract_domain() {
        assert_eq!(
            UrlParser::extract_domain("https://www.example.com/path"),
            Some("example.com".to_string())
        );
        
        assert_eq!(
            UrlParser::extract_domain("http://subdomain.example.org"),
            Some("subdomain.example.org".to_string())
        );
        
        assert_eq!(
            UrlParser::extract_domain("example.net"),
            Some("example.net".to_string())
        );
    }
}