use regex::Regex;

pub struct ParsedUrl {
    pub protocol: String,
    pub host: String,
    pub path: String,
}

pub fn parse_url(url: &str) -> Option<ParsedUrl> {
    let re = Regex::new(r"^(?P<protocol>https?|ftp)://(?P<host>[^/]+)(?P<path>/.*)?$").unwrap();
    let caps = re.captures(url)?;

    let protocol = caps.name("protocol")?.as_str().to_string();
    let host = caps.name("host")?.as_str().to_string();
    let path = caps.name("path").map_or("/", |m| m.as_str()).to_string();

    Some(ParsedUrl { protocol, host, path })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_http_url() {
        let parsed = parse_url("http://example.com/path/to/resource").unwrap();
        assert_eq!(parsed.protocol, "http");
        assert_eq!(parsed.host, "example.com");
        assert_eq!(parsed.path, "/path/to/resource");
    }

    #[test]
    fn test_parse_https_url_without_path() {
        let parsed = parse_url("https://api.github.com").unwrap();
        assert_eq!(parsed.protocol, "https");
        assert_eq!(parsed.host, "api.github.com");
        assert_eq!(parsed.path, "/");
    }

    #[test]
    fn test_parse_invalid_url() {
        let parsed = parse_url("not-a-valid-url");
        assert!(parsed.is_none());
    }
}