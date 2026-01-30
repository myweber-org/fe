use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_domain(url: &str) -> Option<String> {
        let url = url.trim();
        if url.is_empty() {
            return None;
        }

        let url_lower = url.to_lowercase();
        let prefixes = ["http://", "https://", "ftp://", "//"];

        let mut start = 0;
        for prefix in prefixes.iter() {
            if url_lower.starts_with(prefix) {
                start = prefix.len();
                break;
            }
        }

        let remaining = &url[start..];
        let domain_end = remaining.find('/').unwrap_or(remaining.len());
        let domain = &remaining[..domain_end];

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
                    let key = parts[0].to_string();
                    let value = parts[1].to_string();
                    params.insert(key, value);
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

        let url_lower = url.to_lowercase();
        let prefixes = ["http://", "https://", "ftp://", "//"];

        let mut start = 0;
        for prefix in prefixes.iter() {
            if url_lower.starts_with(prefix) {
                start = prefix.len();
                break;
            }
        }

        let remaining = &url[start..];
        if let Some(slash_pos) = remaining.find('/') {
            let path_start = slash_pos;
            let path_and_query = &remaining[path_start..];
            
            if let Some(query_pos) = path_and_query.find('?') {
                Some(path_and_query[..query_pos].to_string())
            } else {
                Some(path_and_query.to_string())
            }
        } else {
            Some("/".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_domain() {
        assert_eq!(UrlParser::parse_domain("https://example.com/path"), Some("example.com".to_string()));
        assert_eq!(UrlParser::parse_domain("http://sub.domain.co.uk/"), Some("sub.domain.co.uk".to_string()));
        assert_eq!(UrlParser::parse_domain("ftp://files.server.net/file.txt"), Some("files.server.net".to_string()));
        assert_eq!(UrlParser::parse_domain("//cdn.provider.com/asset.js"), Some("cdn.provider.com".to_string()));
        assert_eq!(UrlParser::parse_domain("invalid-url"), Some("invalid-url".to_string()));
        assert_eq!(UrlParser::parse_domain(""), None);
    }

    #[test]
    fn test_parse_query_params() {
        let params = UrlParser::parse_query_params("https://example.com/search?q=rust&lang=en&sort=desc");
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("lang"), Some(&"en".to_string()));
        assert_eq!(params.get("sort"), Some(&"desc".to_string()));
        
        let empty_params = UrlParser::parse_query_params("https://example.com/page");
        assert!(empty_params.is_empty());
    }

    #[test]
    fn test_extract_path() {
        assert_eq!(UrlParser::extract_path("https://example.com/api/users"), Some("/api/users".to_string()));
        assert_eq!(UrlParser::extract_path("http://test.com/path/to/resource?param=value"), Some("/path/to/resource".to_string()));
        assert_eq!(UrlParser::extract_path("ftp://server.com/"), Some("/".to_string()));
        assert_eq!(UrlParser::extract_path("//cdn.com/static/js/app.js"), Some("/static/js/app.js".to_string()));
        assert_eq!(UrlParser::extract_path("example.com"), Some("/".to_string()));
        assert_eq!(UrlParser::extract_path(""), None);
    }
}