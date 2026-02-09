use std::collections::HashMap;

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

pub fn build_query_string(params: &HashMap<String, String>) -> String {
    let mut pairs: Vec<String> = Vec::new();
    
    for (key, value) in params {
        pairs.push(format!("{}={}", key, value));
    }
    
    pairs.join("&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_query_string() {
        let query = "name=john&age=30&city=new+york";
        let params = parse_query_string(query);
        
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
        assert_eq!(params.get("city"), Some(&"new+york".to_string()));
        assert_eq!(params.len(), 3);
    }

    #[test]
    fn test_parse_empty_query() {
        let params = parse_query_string("");
        assert!(params.is_empty());
    }

    #[test]
    fn test_build_query_string() {
        let mut params = HashMap::new();
        params.insert("name".to_string(), "alice".to_string());
        params.insert("score".to_string(), "95".to_string());
        
        let query = build_query_string(&params);
        assert!(query.contains("name=alice"));
        assert!(query.contains("score=95"));
        assert_eq!(query.matches('&').count(), 1);
    }

    #[test]
    fn test_round_trip() {
        let original = "key1=value1&key2=value2&key3=value3";
        let params = parse_query_string(original);
        let reconstructed = build_query_string(&params);
        
        let params1 = parse_query_string(original);
        let params2 = parse_query_string(&reconstructed);
        assert_eq!(params1, params2);
    }
}
use std::collections::HashMap;

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
    
    pub fn extract_domain(url: &str) -> Option<String> {
        let url_lower = url.to_lowercase();
        
        if url_lower.starts_with("http://") || url_lower.starts_with("https://") {
            let after_protocol = if url_lower.starts_with("http://") {
                &url[7..]
            } else {
                &url[8..]
            };
            
            if let Some(slash_pos) = after_protocol.find('/') {
                return Some(after_protocol[..slash_pos].to_string());
            }
            return Some(after_protocol.to_string());
        }
        
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_query_parsing() {
        let url = "https://example.com/search?q=rust&page=2&sort=desc";
        let params = UrlParser::parse_query_string(url);
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("page"), Some(&"2".to_string()));
        assert_eq!(params.get("sort"), Some(&"desc".to_string()));
    }
    
    #[test]
    fn test_domain_extraction() {
        let url = "https://api.github.com/users/rust-lang";
        let domain = UrlParser::extract_domain(url);
        
        assert_eq!(domain, Some("api.github.com".to_string()));
    }
}