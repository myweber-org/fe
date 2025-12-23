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
}