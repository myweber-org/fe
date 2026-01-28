use regex::Regex;

pub struct ParsedUrl {
    pub protocol: String,
    pub domain: String,
    pub path: String,
}

pub fn parse_url(url: &str) -> Option<ParsedUrl> {
    let re = Regex::new(r"^(?P<protocol>https?|ftp)://(?P<domain>[^/]+)(?P<path>/.*)?$").unwrap();
    
    re.captures(url).map(|caps| {
        let protocol = caps.name("protocol").map_or("", |m| m.as_str()).to_string();
        let domain = caps.name("domain").map_or("", |m| m.as_str()).to_string();
        let path = caps.name("path").map_or("/", |m| m.as_str()).to_string();
        
        ParsedUrl { protocol, domain, path }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_url() {
        let result = parse_url("https://example.com/path/to/resource");
        assert!(result.is_some());
        let parsed = result.unwrap();
        assert_eq!(parsed.protocol, "https");
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.path, "/path/to/resource");
    }

    #[test]
    fn test_parse_url_without_path() {
        let result = parse_url("http://example.com");
        assert!(result.is_some());
        let parsed = result.unwrap();
        assert_eq!(parsed.protocol, "http");
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.path, "/");
    }

    #[test]
    fn test_parse_invalid_url() {
        let result = parse_url("invalid-url");
        assert!(result.is_none());
    }
}use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_domain(url: &str) -> Option<String> {
        let url = url.trim();
        if url.is_empty() {
            return None;
        }

        let after_protocol = if let Some(pos) = url.find("://") {
            &url[pos + 3..]
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
        
        let query_start = url.find('?');
        if query_start.is_none() {
            return params;
        }

        let query_string = &url[query_start.unwrap() + 1..];
        
        for pair in query_string.split('&') {
            let parts: Vec<&str> = pair.split('=').collect();
            if parts.len() == 2 {
                params.insert(parts[0].to_string(), parts[1].to_string());
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
        assert_eq!(UrlParser::parse_domain("https://example.com/path"), Some("example.com".to_string()));
        assert_eq!(UrlParser::parse_domain("http://sub.example.com:8080/"), Some("sub.example.com:8080".to_string()));
        assert_eq!(UrlParser::parse_domain("example.com"), Some("example.com".to_string()));
        assert_eq!(UrlParser::parse_domain(""), None);
    }

    #[test]
    fn test_parse_query_params() {
        let params = UrlParser::parse_query_params("https://example.com?name=john&age=30&city=nyc");
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
        assert_eq!(params.get("city"), Some(&"nyc".to_string()));
        assert_eq!(params.get("country"), None);
    }

    #[test]
    fn test_parse_full() {
        let (domain, params) = UrlParser::parse("https://api.example.com/v1/users?limit=10&offset=5&sort=desc");
        assert_eq!(domain, Some("api.example.com".to_string()));
        assert_eq!(params.get("limit"), Some(&"10".to_string()));
        assert_eq!(params.get("offset"), Some(&"5".to_string()));
        assert_eq!(params.get("sort"), Some(&"desc".to_string()));
    }
}
use regex::Regex;

pub struct ParsedUrl {
    pub scheme: String,
    pub host: String,
    pub path: String,
}

pub fn parse_url(url: &str) -> Option<ParsedUrl> {
    let re = Regex::new(r"^(?P<scheme>https?|ftp)://(?P<host>[^/]+)(?P<path>/.*)?$").unwrap();
    let caps = re.captures(url)?;

    Some(ParsedUrl {
        scheme: caps["scheme"].to_string(),
        host: caps["host"].to_string(),
        path: caps.name("path").map_or("/", |m| m.as_str()).to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_http_url() {
        let parsed = parse_url("http://example.com/foo/bar").unwrap();
        assert_eq!(parsed.scheme, "http");
        assert_eq!(parsed.host, "example.com");
        assert_eq!(parsed.path, "/foo/bar");
    }

    #[test]
    fn test_parse_https_url_without_path() {
        let parsed = parse_url("https://rust-lang.org").unwrap();
        assert_eq!(parsed.scheme, "https");
        assert_eq!(parsed.host, "rust-lang.org");
        assert_eq!(parsed.path, "/");
    }

    #[test]
    fn test_parse_invalid_url() {
        let parsed = parse_url("not-a-valid-url");
        assert!(parsed.is_none());
    }
}