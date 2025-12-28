use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct ParsedUrl {
    pub protocol: String,
    pub host: String,
    pub port: Option<u16>,
    pub path: String,
    pub query_params: HashMap<String, String>,
    pub fragment: Option<String>,
}

pub fn parse_url(url: &str) -> Result<ParsedUrl, String> {
    let mut protocol = String::new();
    let mut host = String::new();
    let mut port = None;
    let mut path = String::new();
    let mut query_params = HashMap::new();
    let mut fragment = None;

    let url_lower = url.to_lowercase();
    let mut chars = url_lower.chars().peekable();

    while let Some(c) = chars.peek() {
        if *c == ':' && protocol.is_empty() {
            protocol = url[..url_lower.find(':').unwrap()].to_string();
            chars.nth(protocol.len());
            if chars.next() != Some('/') || chars.next() != Some('/') {
                return Err("Invalid URL format".to_string());
            }
            continue;
        }

        if host.is_empty() && !protocol.is_empty() {
            let mut host_builder = String::new();
            while let Some(&c) = chars.peek() {
                if c == ':' || c == '/' || c == '?' || c == '#' {
                    break;
                }
                host_builder.push(c);
                chars.next();
            }
            host = host_builder;

            if let Some(&':') = chars.peek() {
                chars.next();
                let mut port_builder = String::new();
                while let Some(&c) = chars.peek() {
                    if !c.is_ascii_digit() {
                        break;
                    }
                    port_builder.push(c);
                    chars.next();
                }
                if !port_builder.is_empty() {
                    port = Some(port_builder.parse().map_err(|_| "Invalid port")?);
                }
            }
            continue;
        }

        if let Some(&'/') = chars.peek() {
            let mut path_builder = String::new();
            while let Some(&c) = chars.peek() {
                if c == '?' || c == '#' {
                    break;
                }
                path_builder.push(c);
                chars.next();
            }
            path = path_builder;
            continue;
        }

        if let Some(&'?') = chars.peek() {
            chars.next();
            let mut query_string = String::new();
            while let Some(&c) = chars.peek() {
                if c == '#' {
                    break;
                }
                query_string.push(c);
                chars.next();
            }

            for pair in query_string.split('&') {
                let mut parts = pair.split('=');
                if let Some(key) = parts.next() {
                    let value = parts.next().unwrap_or("");
                    query_params.insert(key.to_string(), value.to_string());
                }
            }
            continue;
        }

        if let Some(&'#') = chars.peek() {
            chars.next();
            let mut fragment_builder = String::new();
            while let Some(&c) = chars.peek() {
                fragment_builder.push(c);
                chars.next();
            }
            fragment = Some(fragment_builder);
            continue;
        }

        chars.next();
    }

    if protocol.is_empty() || host.is_empty() {
        return Err("Invalid URL: missing protocol or host".to_string());
    }

    Ok(ParsedUrl {
        protocol,
        host,
        port,
        path,
        query_params,
        fragment,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_url() {
        let url = "https://example.com/path/to/resource";
        let parsed = parse_url(url).unwrap();
        assert_eq!(parsed.protocol, "https");
        assert_eq!(parsed.host, "example.com");
        assert_eq!(parsed.port, None);
        assert_eq!(parsed.path, "/path/to/resource");
        assert!(parsed.query_params.is_empty());
        assert_eq!(parsed.fragment, None);
    }

    #[test]
    fn test_parse_url_with_port_and_query() {
        let url = "http://localhost:8080/api?user=john&page=2";
        let parsed = parse_url(url).unwrap();
        assert_eq!(parsed.protocol, "http");
        assert_eq!(parsed.host, "localhost");
        assert_eq!(parsed.port, Some(8080));
        assert_eq!(parsed.path, "/api");
        assert_eq!(parsed.query_params.get("user"), Some(&"john".to_string()));
        assert_eq!(parsed.query_params.get("page"), Some(&"2".to_string()));
        assert_eq!(parsed.fragment, None);
    }

    #[test]
    fn test_parse_url_with_fragment() {
        let url = "https://docs.rs/regex/1.5.4/regex/#syntax";
        let parsed = parse_url(url).unwrap();
        assert_eq!(parsed.protocol, "https");
        assert_eq!(parsed.host, "docs.rs");
        assert_eq!(parsed.path, "/regex/1.5.4/regex/");
        assert_eq!(parsed.fragment, Some("syntax".to_string()));
    }
}