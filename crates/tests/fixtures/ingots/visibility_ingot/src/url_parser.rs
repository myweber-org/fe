use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_domain(url: &str) -> Option<String> {
        let url = url.trim();
        if url.is_empty() {
            return None;
        }

        let url_lower = url.to_lowercase();
        let prefixes = ["http://", "https://", "ftp://", "www."];
        
        let mut processed_url = url_lower.as_str();
        for prefix in prefixes.iter() {
            if processed_url.starts_with(prefix) {
                processed_url = &processed_url[prefix.len()..];
                break;
            }
        }

        let domain_end = processed_url.find('/').unwrap_or(processed_url.len());
        let domain = &processed_url[..domain_end];
        
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
                    params.insert(
                        parts[0].to_string(),
                        parts[1].to_string()
                    );
                }
            }
        }
        
        params
    }

    pub fn extract_path(url: &str) -> Option<String> {
        let url = url.trim();
        if url.is_empty() {
            return None;
        }

        let protocol_end = if let Some(pos) = url.find("://") {
            pos + 3
        } else {
            0
        };

        let url_without_protocol = &url[protocol_end..];
        let domain_end = url_without_protocol.find('/').unwrap_or(url_without_protocol.len());
        
        if domain_end == url_without_protocol.len() {
            return Some("/".to_string());
        }

        let path_start = domain_end;
        let path_and_query = &url_without_protocol[path_start..];
        
        let query_start = path_and_query.find('?').unwrap_or(path_and_query.len());
        let path = &path_and_query[..query_start];
        
        if path.is_empty() {
            Some("/".to_string())
        } else {
            Some(path.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_domain() {
        assert_eq!(
            UrlParser::parse_domain("https://www.example.com/path"),
            Some("example.com".to_string())
        );
        assert_eq!(
            UrlParser::parse_domain("http://subdomain.example.co.uk:8080"),
            Some("subdomain.example.co.uk:8080".to_string())
        );
        assert_eq!(UrlParser::parse_domain("invalid-url"), None);
    }

    #[test]
    fn test_parse_query_params() {
        let url = "https://example.com/search?q=rust&lang=en&sort=desc";
        let params = UrlParser::parse_query_params(url);
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("lang"), Some(&"en".to_string()));
        assert_eq!(params.get("sort"), Some(&"desc".to_string()));
        assert_eq!(params.get("missing"), None);
    }

    #[test]
    fn test_extract_path() {
        assert_eq!(
            UrlParser::extract_path("https://example.com/api/v1/users"),
            Some("/api/v1/users".to_string())
        );
        assert_eq!(
            UrlParser::extract_path("https://example.com/"),
            Some("/".to_string())
        );
        assert_eq!(
            UrlParser::extract_path("https://example.com"),
            Some("/".to_string())
        );
        assert_eq!(
            UrlParser::extract_path("https://example.com/search?q=test"),
            Some("/search".to_string())
        );
    }
}