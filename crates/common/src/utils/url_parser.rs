use std::collections::HashMap;

pub struct QueryParser;

impl QueryParser {
    pub fn parse(query: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if query.is_empty() {
            return params;
        }

        for pair in query.split('&') {
            let mut parts = pair.splitn(2, '=');
            if let Some(key) = parts.next() {
                if let Some(value) = parts.next() {
                    params.insert(key.to_string(), value.to_string());
                } else {
                    params.insert(key.to_string(), String::new());
                }
            }
        }
        
        params
    }
    
    pub fn parse_url(url: &str) -> Option<HashMap<String, String>> {
        let parts: Vec<&str> = url.split('?').collect();
        if parts.len() == 2 {
            Some(Self::parse(parts[1]))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_query() {
        let query = "name=john&age=30";
        let params = QueryParser::parse(query);
        
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
    }

    #[test]
    fn test_parse_empty_value() {
        let query = "flag=&empty";
        let params = QueryParser::parse(query);
        
        assert_eq!(params.get("flag"), Some(&"".to_string()));
        assert_eq!(params.get("empty"), Some(&"".to_string()));
    }

    #[test]
    fn test_parse_url_with_query() {
        let url = "https://example.com/search?q=rust&sort=desc";
        let params = QueryParser::parse_url(url).unwrap();
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("sort"), Some(&"desc".to_string()));
    }

    #[test]
    fn test_parse_url_without_query() {
        let url = "https://example.com/path";
        let params = QueryParser::parse_url(url);
        
        assert!(params.is_none());
    }
}