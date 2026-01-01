use regex::Regex;

pub struct ParsedUrl {
    pub protocol: String,
    pub host: String,
    pub path: String,
}

pub fn parse_url(url: &str) -> Option<ParsedUrl> {
    let re = Regex::new(r"^(?P<protocol>https?|ftp)://(?P<host>[^/]+)(?P<path>/.*)?$").unwrap();
    let captures = re.captures(url)?;

    let protocol = captures.name("protocol")?.as_str().to_string();
    let host = captures.name("host")?.as_str().to_string();
    let path = captures
        .name("path")
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| "/".to_string());

    Some(ParsedUrl {
        protocol,
        host,
        path,
    })
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
        let parsed = parse_url("https://example.com").unwrap();
        assert_eq!(parsed.protocol, "https");
        assert_eq!(parsed.host, "example.com");
        assert_eq!(parsed.path, "/");
    }

    #[test]
    fn test_parse_ftp_url() {
        let parsed = parse_url("ftp://files.example.com/pub/data.txt").unwrap();
        assert_eq!(parsed.protocol, "ftp");
        assert_eq!(parsed.host, "files.example.com");
        assert_eq!(parsed.path, "/pub/data.txt");
    }

    #[test]
    fn test_invalid_url_returns_none() {
        let result = parse_url("not-a-valid-url");
        assert!(result.is_none());
    }
}