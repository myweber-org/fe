
use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_query_string(url: &str) -> Option<HashMap<String, String>> {
        let query_start = url.find('?')?;
        let query_str = &url[query_start + 1..];
        
        if query_str.is_empty() {
            return Some(HashMap::new());
        }
        
        let mut params = HashMap::new();
        
        for pair in query_str.split('&') {
            let mut parts = pair.splitn(2, '=');
            if let Some(key) = parts.next() {
                let value = parts.next().unwrap_or("");
                params.insert(key.to_string(), value.to_string());
            }
        }
        
        Some(params)
    }
    
    pub fn extract_domain(url: &str) -> Option<String> {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_query_string() {
        let url = "https://example.com/search?q=rust&lang=en&sort=desc";
        let params = UrlParser::parse_query_string(url).unwrap();
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("lang"), Some(&"en".to_string()));
        assert_eq!(params.get("sort"), Some(&"desc".to_string()));
        assert_eq!(params.len(), 3);
    }
    
    #[test]
    fn test_extract_domain() {
        assert_eq!(
            UrlParser::extract_domain("https://www.example.com/path"),
            Some("www.example.com".to_string())
        );
        
        assert_eq!(
            UrlParser::extract_domain("example.com/page"),
            Some("example.com".to_string())
        );
        
        assert_eq!(
            UrlParser::extract_domain("invalid://"),
            None
        );
    }
}use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct ParsedUrl {
    pub protocol: String,
    pub host: String,
    pub port: Option<u16>,
    pub path: String,
    pub query_params: HashMap<String, String>,
}

impl ParsedUrl {
    pub fn parse(url: &str) -> Result<Self, String> {
        let mut protocol = String::new();
        let mut host = String::new();
        let mut port = None;
        let mut path = String::new();
        let mut query_params = HashMap::new();

        let parts: Vec<&str> = url.split("://").collect();
        if parts.len() != 2 {
            return Err("Invalid URL format".to_string());
        }

        protocol = parts[0].to_string();
        let rest = parts[1];

        let host_path_split: Vec<&str> = rest.splitn(2, '/').collect();
        let authority = host_path_split[0];
        let path_and_query = if host_path_split.len() > 1 {
            format!("/{}", host_path_split[1])
        } else {
            "/".to_string()
        };

        let host_port_split: Vec<&str> = authority.split(':').collect();
        host = host_port_split[0].to_string();

        if host_port_split.len() == 2 {
            if let Ok(p) = host_port_split[1].parse::<u16>() {
                port = Some(p);
            } else {
                return Err("Invalid port number".to_string());
            }
        }

        let path_query_split: Vec<&str> = path_and_query.splitn(2, '?').collect();
        path = path_query_split[0].to_string();

        if path_query_split.len() == 2 {
            for pair in path_query_split[1].split('&') {
                let kv: Vec<&str> = pair.splitn(2, '=').collect();
                if kv.len() == 2 {
                    query_params.insert(kv[0].to_string(), kv[1].to_string());
                }
            }
        }

        Ok(ParsedUrl {
            protocol,
            host,
            port,
            path,
            query_params,
        })
    }

    pub fn build_url(&self) -> String {
        let mut url = format!("{}://{}", self.protocol, self.host);
        
        if let Some(port) = self.port {
            url.push_str(&format!(":{}", port));
        }
        
        url.push_str(&self.path);
        
        if !self.query_params.is_empty() {
            url.push('?');
            let query_string: Vec<String> = self.query_params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            url.push_str(&query_string.join("&"));
        }
        
        url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_url() {
        let url = "https://example.com/path/to/resource";
        let parsed = ParsedUrl::parse(url).unwrap();
        
        assert_eq!(parsed.protocol, "https");
        assert_eq!(parsed.host, "example.com");
        assert_eq!(parsed.port, None);
        assert_eq!(parsed.path, "/path/to/resource");
        assert!(parsed.query_params.is_empty());
    }

    #[test]
    fn test_parse_url_with_port_and_query() {
        let url = "http://localhost:8080/api/data?page=1&limit=10";
        let parsed = ParsedUrl::parse(url).unwrap();
        
        assert_eq!(parsed.protocol, "http");
        assert_eq!(parsed.host, "localhost");
        assert_eq!(parsed.port, Some(8080));
        assert_eq!(parsed.path, "/api/data");
        assert_eq!(parsed.query_params.get("page"), Some(&"1".to_string()));
        assert_eq!(parsed.query_params.get("limit"), Some(&"10".to_string()));
    }

    #[test]
    fn test_build_url() {
        let mut query_params = HashMap::new();
        query_params.insert("sort".to_string(), "desc".to_string());
        query_params.insert("filter".to_string(), "active".to_string());
        
        let parsed = ParsedUrl {
            protocol: "https".to_string(),
            host: "api.example.com".to_string(),
            port: Some(443),
            path: "/v1/users".to_string(),
            query_params,
        };
        
        let built_url = parsed.build_url();
        assert!(built_url.starts_with("https://api.example.com:443/v1/users?"));
        assert!(built_url.contains("sort=desc"));
        assert!(built_url.contains("filter=active"));
    }

    #[test]
    fn test_parse_invalid_url() {
        let url = "not-a-valid-url";
        let result = ParsedUrl::parse(url);
        assert!(result.is_err());
    }
}