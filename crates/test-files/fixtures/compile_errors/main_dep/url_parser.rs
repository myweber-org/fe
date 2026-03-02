use regex::Regex;
use std::collections::HashMap;

pub struct ParsedUrl {
    pub protocol: String,
    pub domain: String,
    pub path: String,
    pub query_params: HashMap<String, String>,
    pub fragment: Option<String>,
}

impl ParsedUrl {
    pub fn new(url: &str) -> Result<Self, String> {
        let url_pattern = Regex::new(r"^(?P<protocol>https?://)?(?P<domain>[^/?#]+)(?P<path>/[^?#]*)?(?P<query>\?[^#]*)?(?P<fragment>#.*)?$")
            .map_err(|e| format!("Regex compilation failed: {}", e))?;

        let captures = url_pattern.captures(url)
            .ok_or_else(|| "Invalid URL format".to_string())?;

        let protocol = captures.name("protocol")
            .map(|m| m.as_str().trim_end_matches("://").to_string())
            .unwrap_or_else(|| "https".to_string());

        let domain = captures.name("domain")
            .map(|m| m.as_str().to_string())
            .ok_or_else(|| "Domain not found in URL".to_string())?;

        let path = captures.name("path")
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "/".to_string());

        let query_params = captures.name("query")
            .map(|q| Self::parse_query_params(q.as_str()))
            .unwrap_or_else(HashMap::new);

        let fragment = captures.name("fragment")
            .map(|m| m.as_str()[1..].to_string());

        Ok(ParsedUrl {
            protocol,
            domain,
            path,
            query_params,
            fragment,
        })
    }

    fn parse_query_params(query_str: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        let query_part = &query_str[1..];
        
        for pair in query_part.split('&') {
            let mut parts = pair.split('=');
            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                params.insert(key.to_string(), value.to_string());
            }
        }
        params
    }

    pub fn get_root_domain(&self) -> Option<String> {
        let parts: Vec<&str> = self.domain.split('.').collect();
        if parts.len() >= 2 {
            Some(format!("{}.{}", parts[parts.len()-2], parts[parts.len()-1]))
        } else {
            None
        }
    }

    pub fn to_string(&self) -> String {
        let mut result = format!("{}://{}{}", self.protocol, self.domain, self.path);
        
        if !self.query_params.is_empty() {
            result.push('?');
            let query_parts: Vec<String> = self.query_params
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
}

pub fn is_valid_url(url: &str) -> bool {
    ParsedUrl::new(url).is_ok()
}

pub fn extract_domain(url: &str) -> Option<String> {
    ParsedUrl::new(url).ok().map(|parsed| parsed.domain)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_url_parsing() {
        let url = "https://example.com/path/to/resource?key1=value1&key2=value2#section";
        let parsed = ParsedUrl::new(url).unwrap();
        
        assert_eq!(parsed.protocol, "https");
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.path, "/path/to/resource");
        assert_eq!(parsed.query_params.get("key1"), Some(&"value1".to_string()));
        assert_eq!(parsed.query_params.get("key2"), Some(&"value2".to_string()));
        assert_eq!(parsed.fragment, Some("section".to_string()));
    }

    #[test]
    fn test_url_without_protocol() {
        let url = "example.com/api/data";
        let parsed = ParsedUrl::new(url).unwrap();
        
        assert_eq!(parsed.protocol, "https");
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.path, "/api/data");
    }

    #[test]
    fn test_root_domain_extraction() {
        let url = "https://subdomain.example.co.uk/path";
        let parsed = ParsedUrl::new(url).unwrap();
        
        assert_eq!(parsed.get_root_domain(), Some("example.co.uk".to_string()));
    }

    #[test]
    fn test_invalid_url() {
        let url = "not-a-valid-url";
        let result = ParsedUrl::new(url);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_url_reconstruction() {
        let original = "https://example.com/test?param=value#frag";
        let parsed = ParsedUrl::new(original).unwrap();
        
        assert_eq!(parsed.to_string(), original);
    }
}