use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct ParsedUrl {
    pub protocol: String,
    pub domain: String,
    pub path: String,
    pub query_params: HashMap<String, String>,
    pub fragment: Option<String>,
}

impl ParsedUrl {
    pub fn parse(url: &str) -> Result<Self, String> {
        let mut protocol = String::new();
        let mut domain = String::new();
        let mut path = String::new();
        let mut query_params = HashMap::new();
        let mut fragment = None;

        let mut remaining = url;

        if let Some(proto_end) = remaining.find("://") {
            protocol = remaining[..proto_end].to_string();
            remaining = &remaining[proto_end + 3..];
        }

        let path_start = remaining.find('/').unwrap_or(remaining.len());
        domain = remaining[..path_start].to_string();
        remaining = &remaining[path_start..];

        let fragment_start = remaining.find('#');
        let query_start = remaining.find('?');

        let query_fragment_start = match (query_start, fragment_start) {
            (Some(q), Some(f)) => Some(q.min(f)),
            (Some(q), None) => Some(q),
            (None, Some(f)) => Some(f),
            (None, None) => None,
        };

        if let Some(start) = query_fragment_start {
            path = remaining[..start].to_string();
            remaining = &remaining[start..];
        } else {
            path = remaining.to_string();
            remaining = "";
        }

        if remaining.starts_with('?') {
            let fragment_start = remaining.find('#');
            let query_part = if let Some(frag_pos) = fragment_start {
                fragment = Some(remaining[frag_pos + 1..].to_string());
                &remaining[1..frag_pos]
            } else {
                &remaining[1..]
            };

            for pair in query_part.split('&') {
                if pair.is_empty() {
                    continue;
                }
                let mut parts = pair.splitn(2, '=');
                if let Some(key) = parts.next() {
                    let value = parts.next().unwrap_or("").to_string();
                    query_params.insert(key.to_string(), value);
                }
            }
        } else if remaining.starts_with('#') {
            fragment = Some(remaining[1..].to_string());
        }

        if domain.is_empty() {
            return Err("Domain cannot be empty".to_string());
        }

        Ok(ParsedUrl {
            protocol,
            domain,
            path,
            query_params,
            fragment,
        })
    }

    pub fn get_query_param(&self, key: &str) -> Option<&String> {
        self.query_params.get(key)
    }

    pub fn has_fragment(&self) -> bool {
        self.fragment.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full_url() {
        let url = "https://example.com/path/to/resource?param1=value1&param2=value2#section";
        let parsed = ParsedUrl::parse(url).unwrap();

        assert_eq!(parsed.protocol, "https");
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.path, "/path/to/resource");
        assert_eq!(parsed.get_query_param("param1"), Some(&"value1".to_string()));
        assert_eq!(parsed.get_query_param("param2"), Some(&"value2".to_string()));
        assert_eq!(parsed.fragment, Some("section".to_string()));
    }

    #[test]
    fn test_parse_url_no_protocol() {
        let url = "example.com/path";
        let parsed = ParsedUrl::parse(url).unwrap();

        assert_eq!(parsed.protocol, "");
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.path, "/path");
        assert!(parsed.query_params.is_empty());
        assert_eq!(parsed.fragment, None);
    }

    #[test]
    fn test_parse_url_empty_domain() {
        let url = ":///path";
        let result = ParsedUrl::parse(url);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_url_only_domain() {
        let url = "example.com";
        let parsed = ParsedUrl::parse(url).unwrap();

        assert_eq!(parsed.protocol, "");
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.path, "");
        assert!(parsed.query_params.is_empty());
        assert_eq!(parsed.fragment, None);
    }
}