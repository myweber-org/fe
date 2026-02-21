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
        let url = url.trim();
        if url.is_empty() {
            return None;
        }

        let url = if !url.starts_with("http") {
            format!("https://{}", url)
        } else {
            url.to_string()
        };

        url::Url::parse(&url)
            .ok()
            .and_then(|parsed| parsed.host_str().map(|host| host.to_string()))
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
        assert_eq!(params.get("country"), None);
    }

    #[test]
    fn test_empty_query_string() {
        let params = UrlParser::parse_query_string("");
        assert!(params.is_empty());
    }

    #[test]
    fn test_extract_domain() {
        assert_eq!(
            UrlParser::extract_domain("https://example.com/path"),
            Some("example.com".to_string())
        );
        assert_eq!(
            UrlParser::extract_domain("example.com"),
            Some("example.com".to_string())
        );
        assert_eq!(
            UrlParser::extract_domain("http://sub.domain.co.uk"),
            Some("sub.domain.co.uk".to_string())
        );
    }

    #[test]
    fn test_invalid_url() {
        assert_eq!(UrlParser::extract_domain(""), None);
        assert_eq!(UrlParser::extract_domain("://bad"), None);
    }
}