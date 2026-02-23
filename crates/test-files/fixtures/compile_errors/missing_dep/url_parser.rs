use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_query_string(query: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if query.is_empty() {
            return params;
        }

        for pair in query.split('&') {
            let mut parts = pair.splitn(2, '=');
            if let Some(key) = parts.next() {
                let value = parts.next().unwrap_or("");
                params.insert(key.to_string(), value.to_string());
            }
        }
        
        params
    }

    pub fn extract_domain(url: &str) -> Option<String> {
        let url_lower = url.to_lowercase();
        let prefixes = ["http://", "https://", "www."];
        
        let mut processed_url = url_lower.as_str();
        for prefix in &prefixes {
            if processed_url.starts_with(prefix) {
                processed_url = &processed_url[prefix.len()..];
            }
        }

        let domain_end = processed_url.find('/').unwrap_or(processed_url.len());
        let domain = &processed_url[..domain_end];
        
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
        let query = "name=john&age=30&city=new+york";
        let params = UrlParser::parse_query_string(query);
        
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
        assert_eq!(params.get("city"), Some(&"new+york".to_string()));
        assert_eq!(params.get("country"), None);
    }

    #[test]
    fn test_extract_domain() {
        assert_eq!(
            UrlParser::extract_domain("https://www.example.com/path"),
            Some("example.com".to_string())
        );
        assert_eq!(
            UrlParser::extract_domain("http://subdomain.example.co.uk/page"),
            Some("subdomain.example.co.uk".to_string())
        );
        assert_eq!(
            UrlParser::extract_domain("invalid-url"),
            None
        );
    }
}use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_query_string(url: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if let Some(query_start) = url.find('?') {
            let query_string = &url[query_start + 1..];
            
            for pair in query_string.split('&') {
                let mut parts = pair.split('=');
                if let Some(key) = parts.next() {
                    let value = parts.next().unwrap_or("");
                    params.insert(key.to_string(), value.to_string());
                }
            }
        }
        
        params
    }
    
    pub fn get_domain(url: &str) -> Option<String> {
        let url = url.trim_start_matches("http://")
            .trim_start_matches("https://");
        
        if let Some(end) = url.find('/') {
            Some(url[..end].to_string())
        } else {
            Some(url.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_query_string() {
        let url = "https://example.com/search?q=rust&page=2&sort=recent";
        let params = UrlParser::parse_query_string(url);
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("page"), Some(&"2".to_string()));
        assert_eq!(params.get("sort"), Some(&"recent".to_string()));
    }
    
    #[test]
    fn test_get_domain() {
        let url = "https://www.example.com/path/to/resource";
        let domain = UrlParser::get_domain(url).unwrap();
        
        assert_eq!(domain, "www.example.com");
    }
}use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct ParsedUrl {
    pub protocol: String,
    pub domain: String,
    pub path: String,
    pub query_params: HashMap<String, String>,
}

impl ParsedUrl {
    pub fn parse(url: &str) -> Result<Self, String> {
        let parts: Vec<&str> = url.split("://").collect();
        if parts.len() != 2 {
            return Err("Invalid URL format".to_string());
        }

        let protocol = parts[0].to_string();
        let rest = parts[1];

        let domain_path: Vec<&str> = rest.splitn(2, '/').collect();
        let domain = domain_path[0].to_string();

        let path_and_query = if domain_path.len() > 1 {
            domain_path[1]
        } else {
            ""
        };

        let path_query: Vec<&str> = path_and_query.splitn(2, '?').collect();
        let path = if !path_query[0].is_empty() {
            format!("/{}", path_query[0])
        } else {
            "/".to_string()
        };

        let mut query_params = HashMap::new();
        if path_query.len() > 1 {
            for pair in path_query[1].split('&') {
                let kv: Vec<&str> = pair.splitn(2, '=').collect();
                if kv.len() == 2 {
                    query_params.insert(kv[0].to_string(), kv[1].to_string());
                }
            }
        }

        Ok(ParsedUrl {
            protocol,
            domain,
            path,
            query_params,
        })
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
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.path, "/path/to/resource");
        assert!(parsed.query_params.is_empty());
    }

    #[test]
    fn test_parse_url_with_query() {
        let url = "http://test.org/api?key=value&sort=desc";
        let parsed = ParsedUrl::parse(url).unwrap();
        assert_eq!(parsed.protocol, "http");
        assert_eq!(parsed.domain, "test.org");
        assert_eq!(parsed.path, "/api");
        assert_eq!(parsed.query_params.get("key"), Some(&"value".to_string()));
        assert_eq!(parsed.query_params.get("sort"), Some(&"desc".to_string()));
    }

    #[test]
    fn test_parse_url_without_path() {
        let url = "ftp://fileserver.net";
        let parsed = ParsedUrl::parse(url).unwrap();
        assert_eq!(parsed.protocol, "ftp");
        assert_eq!(parsed.domain, "fileserver.net");
        assert_eq!(parsed.path, "/");
    }

    #[test]
    fn test_invalid_url() {
        let url = "not-a-valid-url";
        let result = ParsedUrl::parse(url);
        assert!(result.is_err());
    }
}