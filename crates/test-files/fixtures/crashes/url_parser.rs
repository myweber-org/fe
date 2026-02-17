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
        
        if url_lower.starts_with("http://") || url_lower.starts_with("https://") {
            let without_scheme = if url_lower.starts_with("http://") {
                &url[7..]
            } else {
                &url[8..]
            };
            
            let domain_end = without_scheme.find('/').unwrap_or(without_scheme.len());
            Some(without_scheme[..domain_end].to_string())
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
    }

    #[test]
    fn test_extract_domain() {
        assert_eq!(
            UrlParser::extract_domain("https://example.com/path"),
            Some("example.com".to_string())
        );
        
        assert_eq!(
            UrlParser::extract_domain("http://sub.domain.co.uk/"),
            Some("sub.domain.co.uk".to_string())
        );
        
        assert_eq!(UrlParser::extract_domain("invalid-url"), None);
    }
}