
use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse(url: &str) -> Option<ParsedUrl> {
        let url = url.trim();
        if url.is_empty() {
            return None;
        }

        let (scheme, rest) = Self::extract_scheme(url);
        let (domain, path_and_query) = Self::extract_domain(rest);
        let (path, query_string) = Self::split_path_and_query(path_and_query);
        let query_params = Self::parse_query_string(query_string);

        Some(ParsedUrl {
            scheme: scheme.to_string(),
            domain: domain.to_string(),
            path: path.to_string(),
            query_params,
        })
    }

    fn extract_scheme(url: &str) -> (&str, &str) {
        if let Some(pos) = url.find("://") {
            (&url[..pos], &url[pos + 3..])
        } else {
            ("https", url)
        }
    }

    fn extract_domain(rest: &str) -> (&str, &str) {
        if let Some(pos) = rest.find('/') {
            (&rest[..pos], &rest[pos..])
        } else {
            (rest, "/")
        }
    }

    fn split_path_and_query(path_and_query: &str) -> (&str, Option<&str>) {
        if let Some(pos) = path_and_query.find('?') {
            (&path_and_query[..pos], Some(&path_and_query[pos + 1..]))
        } else {
            (path_and_query, None)
        }
    }

    fn parse_query_string(query_string: Option<&str>) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if let Some(query) = query_string {
            for pair in query.split('&') {
                if let Some(equal_pos) = pair.find('=') {
                    let key = &pair[..equal_pos];
                    let value = &pair[equal_pos + 1..];
                    if !key.is_empty() {
                        params.insert(key.to_string(), value.to_string());
                    }
                }
            }
        }
        
        params
    }
}

pub struct ParsedUrl {
    pub scheme: String,
    pub domain: String,
    pub path: String,
    pub query_params: HashMap<String, String>,
}

impl ParsedUrl {
    pub fn get_query_param(&self, key: &str) -> Option<&String> {
        self.query_params.get(key)
    }
    
    pub fn has_query_params(&self) -> bool {
        !self.query_params.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full_url() {
        let url = "https://example.com/path/to/resource?param1=value1&param2=value2";
        let parsed = UrlParser::parse(url).unwrap();
        
        assert_eq!(parsed.scheme, "https");
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.path, "/path/to/resource");
        assert_eq!(parsed.get_query_param("param1"), Some(&"value1".to_string()));
        assert_eq!(parsed.get_query_param("param2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_parse_url_without_scheme() {
        let url = "example.com/api/data";
        let parsed = UrlParser::parse(url).unwrap();
        
        assert_eq!(parsed.scheme, "https");
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.path, "/api/data");
    }

    #[test]
    fn test_parse_url_without_path() {
        let url = "https://example.com";
        let parsed = UrlParser::parse(url).unwrap();
        
        assert_eq!(parsed.scheme, "https");
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.path, "/");
        assert!(!parsed.has_query_params());
    }

    #[test]
    fn test_parse_empty_url() {
        let parsed = UrlParser::parse("");
        assert!(parsed.is_none());
    }
}