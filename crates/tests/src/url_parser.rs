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
        
        let mut cleaned_url = url_lower.as_str();
        for prefix in prefixes.iter() {
            if cleaned_url.starts_with(prefix) {
                cleaned_url = &cleaned_url[prefix.len()..];
            }
        }

        let domain_end = cleaned_url.find('/').unwrap_or(cleaned_url.len());
        let domain = &cleaned_url[..domain_end];
        
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
    }

    #[test]
    fn test_extract_domain() {
        assert_eq!(
            UrlParser::extract_domain("https://www.example.com/path"),
            Some("example.com".to_string())
        );
        assert_eq!(
            UrlParser::extract_domain("http://sub.domain.co.uk/"),
            Some("sub.domain.co.uk".to_string())
        );
        assert_eq!(
            UrlParser::extract_domain("invalid-url"),
            None
        );
    }
}use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse(url: &str) -> Option<ParsedUrl> {
        let mut parts = url.splitn(2, "://");
        let scheme = parts.next()?.to_string();
        let rest = parts.next()?;
        
        let mut host_path_iter = rest.splitn(2, '/');
        let authority = host_path_iter.next()?.to_string();
        let path = host_path_iter.next().unwrap_or("").to_string();
        
        let mut host_port_iter = authority.splitn(2, ':');
        let host = host_port_iter.next()?.to_string();
        let port = host_port_iter.next().and_then(|p| p.parse().ok());
        
        let mut path_query_iter = path.splitn(2, '?');
        let path_only = path_query_iter.next()?.to_string();
        let query_string = path_query_iter.next().unwrap_or("");
        
        let query_params = Self::parse_query_string(query_string);
        
        Some(ParsedUrl {
            scheme,
            host,
            port,
            path: path_only,
            query_params,
        })
    }
    
    fn parse_query_string(query: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        for pair in query.split('&') {
            let mut kv = pair.splitn(2, '=');
            if let (Some(key), Some(value)) = (kv.next(), kv.next()) {
                if !key.is_empty() {
                    params.insert(key.to_string(), value.to_string());
                }
            }
        }
        
        params
    }
    
    pub fn extract_domain(url: &str) -> Option<String> {
        Self::parse(url).map(|parsed| parsed.host)
    }
    
    pub fn get_query_param(url: &str, key: &str) -> Option<String> {
        Self::parse(url)
            .and_then(|parsed| parsed.query_params.get(key).cloned())
    }
}

pub struct ParsedUrl {
    pub scheme: String,
    pub host: String,
    pub port: Option<u16>,
    pub path: String,
    pub query_params: HashMap<String, String>,
}

impl ParsedUrl {
    pub fn full_path(&self) -> String {
        let mut result = self.path.clone();
        
        if !self.query_params.is_empty() {
            result.push('?');
            let queries: Vec<String> = self.query_params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            result.push_str(&queries.join("&"));
        }
        
        result
    }
    
    pub fn to_string(&self) -> String {
        let mut result = format!("{}://{}", self.scheme, self.host);
        
        if let Some(port) = self.port {
            result.push_str(&format!(":{}", port));
        }
        
        if !self.path.is_empty() {
            result.push('/');
            result.push_str(&self.full_path());
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_basic_url() {
        let url = "https://example.com/path/to/resource";
        let parsed = UrlParser::parse(url).unwrap();
        
        assert_eq!(parsed.scheme, "https");
        assert_eq!(parsed.host, "example.com");
        assert_eq!(parsed.port, None);
        assert_eq!(parsed.path, "path/to/resource");
        assert!(parsed.query_params.is_empty());
    }
    
    #[test]
    fn test_parse_url_with_query() {
        let url = "https://api.example.com:8080/search?q=rust&limit=10&sort=desc";
        let parsed = UrlParser::parse(url).unwrap();
        
        assert_eq!(parsed.scheme, "https");
        assert_eq!(parsed.host, "api.example.com");
        assert_eq!(parsed.port, Some(8080));
        assert_eq!(parsed.path, "search");
        
        assert_eq!(parsed.query_params.get("q"), Some(&"rust".to_string()));
        assert_eq!(parsed.query_params.get("limit"), Some(&"10".to_string()));
        assert_eq!(parsed.query_params.get("sort"), Some(&"desc".to_string()));
        assert_eq!(parsed.query_params.len(), 3);
    }
    
    #[test]
    fn test_extract_domain() {
        assert_eq!(
            UrlParser::extract_domain("https://sub.example.co.uk/path"),
            Some("sub.example.co.uk".to_string())
        );
        
        assert_eq!(
            UrlParser::extract_domain("invalid-url"),
            None
        );
    }
    
    #[test]
    fn test_get_query_param() {
        let url = "https://example.com/page?name=john&age=30&city=new+york";
        
        assert_eq!(
            UrlParser::get_query_param(url, "name"),
            Some("john".to_string())
        );
        
        assert_eq!(
            UrlParser::get_query_param(url, "age"),
            Some("30".to_string())
        );
        
        assert_eq!(
            UrlParser::get_query_param(url, "city"),
            Some("new+york".to_string())
        );
        
        assert_eq!(
            UrlParser::get_query_param(url, "nonexistent"),
            None
        );
    }
    
    #[test]
    fn test_reconstruct_url() {
        let original = "https://api.example.com:3000/data/items?category=books&lang=en";
        let parsed = UrlParser::parse(original).unwrap();
        let reconstructed = parsed.to_string();
        
        assert_eq!(original, reconstructed);
    }
}