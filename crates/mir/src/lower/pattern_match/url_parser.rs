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
    
    pub fn get_param(query: &str, key: &str) -> Option<String> {
        Self::parse(query).get(key).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_query() {
        let result = QueryParser::parse("");
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_single_param() {
        let result = QueryParser::parse("name=john");
        assert_eq!(result.get("name"), Some(&"john".to_string()));
    }

    #[test]
    fn test_parse_multiple_params() {
        let result = QueryParser::parse("name=john&age=30&city=newyork");
        assert_eq!(result.get("name"), Some(&"john".to_string()));
        assert_eq!(result.get("age"), Some(&"30".to_string()));
        assert_eq!(result.get("city"), Some(&"newyork".to_string()));
    }

    #[test]
    fn test_parse_param_without_value() {
        let result = QueryParser::parse("flag&name=test");
        assert_eq!(result.get("flag"), Some(&"".to_string()));
        assert_eq!(result.get("name"), Some(&"test".to_string()));
    }

    #[test]
    fn test_get_param_direct() {
        let query = "search=rust&sort=desc";
        assert_eq!(QueryParser::get_param(query, "search"), Some("rust".to_string()));
        assert_eq!(QueryParser::get_param(query, "sort"), Some("desc".to_string()));
        assert_eq!(QueryParser::get_param(query, "missing"), None);
    }
}