use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct ParsedUrl {
    pub protocol: String,
    pub domain: String,
    pub port: Option<u16>,
    pub path: String,
    pub query_params: HashMap<String, String>,
}

pub fn parse_url(url: &str) -> Result<ParsedUrl, String> {
    let mut protocol = String::new();
    let mut domain = String::new();
    let mut port = None;
    let mut path = String::new();
    let mut query_params = HashMap::new();

    let parts: Vec<&str> = url.split("://").collect();
    if parts.len() != 2 {
        return Err("Invalid URL format".to_string());
    }

    protocol = parts[0].to_string();
    let rest = parts[1];

    let domain_path_split: Vec<&str> = rest.splitn(2, '/').collect();
    let authority = domain_path_split[0];
    let path_and_query = if domain_path_split.len() > 1 {
        domain_path_split[1]
    } else {
        ""
    };

    let authority_parts: Vec<&str> = authority.split(':').collect();
    domain = authority_parts[0].to_string();

    if authority_parts.len() > 1 {
        if let Ok(p) = authority_parts[1].parse::<u16>() {
            port = Some(p);
        } else {
            return Err("Invalid port number".to_string());
        }
    }

    let path_query_split: Vec<&str> = path_and_query.splitn(2, '?').collect();
    path = if !path_query_split[0].is_empty() {
        format!("/{}", path_query_split[0])
    } else {
        "/".to_string()
    };

    if path_query_split.len() > 1 {
        for pair in path_query_split[1].split('&') {
            let kv: Vec<&str> = pair.splitn(2, '=').collect();
            if kv.len() == 2 {
                query_params.insert(kv[0].to_string(), kv[1].to_string());
            }
        }
    }

    Ok(ParsedUrl {
        protocol,
        domain,
        port,
        path,
        query_params,
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
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.port, None);
        assert_eq!(parsed.path, "/path/to/resource");
        assert!(parsed.query_params.is_empty());
    }

    #[test]
    fn test_parse_url_with_port_and_query() {
        let url = "http://localhost:8080/api/data?page=1&limit=20";
        let parsed = parse_url(url).unwrap();
        assert_eq!(parsed.protocol, "http");
        assert_eq!(parsed.domain, "localhost");
        assert_eq!(parsed.port, Some(8080));
        assert_eq!(parsed.path, "/api/data");
        assert_eq!(parsed.query_params.get("page"), Some(&"1".to_string()));
        assert_eq!(parsed.query_params.get("limit"), Some(&"20".to_string()));
    }

    #[test]
    fn test_parse_invalid_url() {
        let url = "not_a_valid_url";
        let result = parse_url(url);
        assert!(result.is_err());
    }
}