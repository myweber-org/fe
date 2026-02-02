use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_url(url: &str) -> Option<ParsedUrl> {
        let url = url.trim();
        if url.is_empty() {
            return None;
        }

        let (scheme, rest) = Self::extract_scheme(url)?;
        let (host, path_and_query) = Self::extract_host(rest)?;
        let (path, query) = Self::extract_path_and_query(path_and_query);

        Some(ParsedUrl {
            scheme: scheme.to_string(),
            host: host.to_string(),
            path: path.to_string(),
            query_params: query,
        })
    }

    fn extract_scheme(url: &str) -> Option<(&str, &str)> {
        if let Some(pos) = url.find("://") {
            let scheme = &url[..pos];
            if scheme.chars().all(|c| c.is_ascii_alphabetic()) {
                return Some((scheme, &url[pos + 3..]));
            }
        }
        None
    }

    fn extract_host(rest: &str) -> Option<(&str, &str)> {
        let end = rest.find('/').unwrap_or(rest.len());
        let host = &rest[..end];
        if !host.is_empty() && host.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-') {
            let remaining = if end == rest.len() { "" } else { &rest[end..] };
            Some((host, remaining))
        } else {
            None
        }
    }

    fn extract_path_and_query(path_and_query: &str) -> (&str, HashMap<String, String>) {
        let query_start = path_and_query.find('?');
        let (path, query_str) = match query_start {
            Some(pos) => (&path_and_query[..pos], &path_and_query[pos + 1..]),
            None => (path_and_query, ""),
        };

        let query_params = Self::parse_query_string(query_str);
        (path, query_params)
    }

    fn parse_query_string(query_str: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        if query_str.is_empty() {
            return params;
        }

        for pair in query_str.split('&') {
            let mut parts = pair.splitn(2, '=');
            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                if !key.is_empty() {
                    params.insert(key.to_string(), value.to_string());
                }
            }
        }
        params
    }
}

pub struct ParsedUrl {
    pub scheme: String,
    pub host: String,
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

    pub fn reconstruct_url(&self) -> String {
        let mut url = format!("{}://{}{}", self.scheme, self.host, self.path);
        
        if self.has_query_params() {
            url.push('?');
            let query_parts: Vec<String> = self.query_params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            url.push_str(&query_parts.join("&"));
        }
        
        url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_url() {
        let url = "https://example.com/path?key1=value1&key2=value2";
        let parsed = UrlParser::parse_url(url).unwrap();
        
        assert_eq!(parsed.scheme, "https");
        assert_eq!(parsed.host, "example.com");
        assert_eq!(parsed.path, "/path");
        assert_eq!(parsed.get_query_param("key1"), Some(&"value1".to_string()));
        assert_eq!(parsed.get_query_param("key2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_parse_url_no_query() {
        let url = "http://api.test.com/resource";
        let parsed = UrlParser::parse_url(url).unwrap();
        
        assert_eq!(parsed.scheme, "http");
        assert_eq!(parsed.host, "api.test.com");
        assert_eq!(parsed.path, "/resource");
        assert!(!parsed.has_query_params());
    }

    #[test]
    fn test_invalid_url() {
        assert!(UrlParser::parse_url("").is_none());
        assert!(UrlParser::parse_url("invalid").is_none());
        assert!(UrlParser::parse_url("://host").is_none());
    }

    #[test]
    fn test_reconstruct_url() {
        let url = "https://example.com/search?q=rust&lang=en";
        let parsed = UrlParser::parse_url(url).unwrap();
        let reconstructed = parsed.reconstruct_url();
        
        assert_eq!(reconstructed, url);
    }
}