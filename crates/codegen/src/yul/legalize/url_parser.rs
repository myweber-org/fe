
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct ParsedUrl {
    pub scheme: String,
    pub host: String,
    pub port: Option<u16>,
    pub path: String,
    pub query_params: HashMap<String, String>,
    pub fragment: Option<String>,
}

impl ParsedUrl {
    pub fn new(url: &str) -> Result<Self, String> {
        if url.trim().is_empty() {
            return Err("URL cannot be empty".to_string());
        }

        let (scheme, rest) = match url.find("://") {
            Some(pos) => {
                let scheme_part = &url[..pos];
                if scheme_part.is_empty() {
                    return Err("Scheme cannot be empty".to_string());
                }
                (scheme_part.to_lowercase(), &url[pos + 3..])
            }
            None => return Err("URL must contain a scheme (e.g., http://)".to_string()),
        };

        let mut host_port = rest;
        let mut path_query_fragment = "";

        if let Some(pos) = host_port.find('/') {
            host_port = &rest[..pos];
            path_query_fragment = &rest[pos..];
        } else if let Some(pos) = host_port.find('?') {
            host_port = &rest[..pos];
            path_query_fragment = &rest[pos..];
        } else if let Some(pos) = host_port.find('#') {
            host_port = &rest[..pos];
            path_query_fragment = &rest[pos..];
        }

        let (host, port) = parse_host_port(host_port)?;

        let (path, query_fragment) = split_path_query_fragment(path_query_fragment);
        let (query, fragment) = split_query_fragment(query_fragment);

        let query_params = parse_query_string(query);

        Ok(ParsedUrl {
            scheme,
            host,
            port,
            path: path.to_string(),
            query_params,
            fragment: fragment.map(|s| s.to_string()),
        })
    }

    pub fn full_path(&self) -> String {
        let mut result = self.path.clone();

        if !self.query_params.is_empty() {
            result.push('?');
            let query_parts: Vec<String> = self
                .query_params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            result.push_str(&query_parts.join("&"));
        }

        if let Some(ref fragment) = self.fragment {
            result.push('#');
            result.push_str(fragment);
        }

        result
    }

    pub fn authority(&self) -> String {
        if let Some(port) = self.port {
            format!("{}:{}", self.host, port)
        } else {
            self.host.clone()
        }
    }
}

fn parse_host_port(host_port: &str) -> Result<(String, Option<u16>), String> {
    if host_port.is_empty() {
        return Err("Host cannot be empty".to_string());
    }

    let (host, port_str) = if let Some(pos) = host_port.find(':') {
        (&host_port[..pos], Some(&host_port[pos + 1..]))
    } else {
        (host_port, None)
    };

    if host.is_empty() {
        return Err("Host cannot be empty".to_string());
    }

    let port = if let Some(port_str) = port_str {
        match port_str.parse::<u16>() {
            Ok(port) => Some(port),
            Err(_) => return Err("Invalid port number".to_string()),
        }
    } else {
        None
    };

    Ok((host.to_string(), port))
}

fn split_path_query_fragment(input: &str) -> (&str, &str) {
    if input.is_empty() {
        return ("/", "");
    }

    if input.starts_with('/') {
        (input, "")
    } else if input.starts_with('?') || input.starts_with('#') {
        ("/", input)
    } else {
        ("/", input)
    }
}

fn split_query_fragment(input: &str) -> (&str, Option<&str>) {
    if input.is_empty() {
        return ("", None);
    }

    if let Some(pos) = input.find('#') {
        (&input[..pos], Some(&input[pos + 1..]))
    } else {
        (input, None)
    }
}

fn parse_query_string(query: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();

    if query.is_empty() || !query.starts_with('?') {
        return params;
    }

    let query_str = &query[1..];
    if query_str.is_empty() {
        return params;
    }

    for pair in query_str.split('&') {
        if let Some(eq_pos) = pair.find('=') {
            let key = &pair[..eq_pos];
            let value = &pair[eq_pos + 1..];
            if !key.is_empty() {
                params.insert(key.to_string(), value.to_string());
            }
        } else if !pair.is_empty() {
            params.insert(pair.to_string(), String::new());
        }
    }

    params
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_url() {
        let url = ParsedUrl::new("https://example.com/path").unwrap();
        assert_eq!(url.scheme, "https");
        assert_eq!(url.host, "example.com");
        assert_eq!(url.port, None);
        assert_eq!(url.path, "/path");
        assert!(url.query_params.is_empty());
        assert_eq!(url.fragment, None);
    }

    #[test]
    fn test_url_with_port() {
        let url = ParsedUrl::new("http://localhost:8080/api").unwrap();
        assert_eq!(url.scheme, "http");
        assert_eq!(url.host, "localhost");
        assert_eq!(url.port, Some(8080));
        assert_eq!(url.path, "/api");
    }

    #[test]
    fn test_url_with_query() {
        let url = ParsedUrl::new("https://api.example.com/search?q=rust&lang=en").unwrap();
        assert_eq!(url.scheme, "https");
        assert_eq!(url.host, "api.example.com");
        assert_eq!(url.path, "/search");
        assert_eq!(url.query_params.get("q"), Some(&"rust".to_string()));
        assert_eq!(url.query_params.get("lang"), Some(&"en".to_string()));
    }

    #[test]
    fn test_url_with_fragment() {
        let url = ParsedUrl::new("https://docs.rs/regex#installation").unwrap();
        assert_eq!(url.scheme, "https");
        assert_eq!(url.host, "docs.rs");
        assert_eq!(url.path, "/regex");
        assert_eq!(url.fragment, Some("installation".to_string()));
    }

    #[test]
    fn test_full_path() {
        let url = ParsedUrl::new("https://example.com/path?key=value#section").unwrap();
        assert_eq!(url.full_path(), "/path?key=value#section");
    }

    #[test]
    fn test_authority() {
        let url = ParsedUrl::new("https://example.com:8443/path").unwrap();
        assert_eq!(url.authority(), "example.com:8443");
    }

    #[test]
    fn test_invalid_urls() {
        assert!(ParsedUrl::new("").is_err());
        assert!(ParsedUrl::new("://example.com").is_err());
        assert!(ParsedUrl::new("http://").is_err());
        assert!(ParsedUrl::new("http://:8080").is_err());
        assert!(ParsedUrl::new("http://example.com:99999").is_err());
    }
}