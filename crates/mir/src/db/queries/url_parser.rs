
use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse(url: &str) -> Result<ParsedUrl, &'static str> {
        if url.is_empty() {
            return Err("Empty URL provided");
        }

        let (scheme, rest) = match url.find("://") {
            Some(pos) => (&url[..pos], &url[pos + 3..]),
            None => ("", url),
        };

        let (host_path, fragment) = match rest.split_once('#') {
            Some((hp, f)) => (hp, Some(f)),
            None => (rest, None),
        };

        let (authority, path_query) = match host_path.split_once('/') {
            Some((auth, pq)) => (auth, Some(pq)),
            None => (host_path, None),
        };

        let (host_port, user_info) = match authority.split_once('@') {
            Some((hi, ui)) => (hi, Some(ui)),
            None => (authority, None),
        };

        let (host, port) = match host_port.split_once(':') {
            Some((h, p)) => (h, Some(p)),
            None => (host_port, None),
        };

        let (path, query) = if let Some(pq) = path_query {
            match pq.split_once('?') {
                Some((p, q)) => (Some(p), Self::parse_query(q)),
                None => (Some(pq), HashMap::new()),
            }
        } else {
            (None, HashMap::new())
        };

        Ok(ParsedUrl {
            scheme: scheme.to_string(),
            user_info: user_info.map(String::from),
            host: host.to_string(),
            port: port.map(String::from),
            path: path.map(String::from),
            query,
            fragment: fragment.map(String::from),
        })
    }

    pub fn extract_domain(url: &str) -> Option<String> {
        Self::parse(url).ok().map(|parsed| parsed.host)
    }

    fn parse_query(query_str: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        for pair in query_str.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                params.insert(key.to_string(), value.to_string());
            }
        }
        
        params
    }
}

pub struct ParsedUrl {
    pub scheme: String,
    pub user_info: Option<String>,
    pub host: String,
    pub port: Option<String>,
    pub path: Option<String>,
    pub query: HashMap<String, String>,
    pub fragment: Option<String>,
}

impl ParsedUrl {
    pub fn get_query_param(&self, key: &str) -> Option<&String> {
        self.query.get(key)
    }
    
    pub fn has_query_params(&self) -> bool {
        !self.query.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full_url() {
        let url = "https://user:pass@example.com:8080/path/to/resource?key1=value1&key2=value2#section";
        let parsed = UrlParser::parse(url).unwrap();
        
        assert_eq!(parsed.scheme, "https");
        assert_eq!(parsed.user_info, Some("user:pass".to_string()));
        assert_eq!(parsed.host, "example.com");
        assert_eq!(parsed.port, Some("8080".to_string()));
        assert_eq!(parsed.path, Some("/path/to/resource".to_string()));
        assert_eq!(parsed.get_query_param("key1"), Some(&"value1".to_string()));
        assert_eq!(parsed.get_query_param("key2"), Some(&"value2".to_string()));
        assert_eq!(parsed.fragment, Some("section".to_string()));
    }

    #[test]
    fn test_extract_domain() {
        assert_eq!(
            UrlParser::extract_domain("https://www.rust-lang.org/learn"),
            Some("www.rust-lang.org".to_string())
        );
        
        assert_eq!(
            UrlParser::extract_domain("invalid-url"),
            Some("invalid-url".to_string())
        );
    }

    #[test]
    fn test_empty_url() {
        assert!(UrlParser::parse("").is_err());
    }
}