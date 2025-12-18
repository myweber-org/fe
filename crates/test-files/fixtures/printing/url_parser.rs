
use std::collections::HashMap;

pub struct QueryParams {
    params: HashMap<String, Vec<String>>,
}

impl QueryParams {
    pub fn from_query_string(query: &str) -> Self {
        let mut params = HashMap::new();
        
        if query.is_empty() {
            return QueryParams { params };
        }
        
        for pair in query.split('&') {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next().unwrap_or("");
            let value = parts.next().unwrap_or("");
            
            if !key.is_empty() {
                params
                    .entry(key.to_string())
                    .or_insert_with(Vec::new)
                    .push(value.to_string());
            }
        }
        
        QueryParams { params }
    }
    
    pub fn get_first(&self, key: &str) -> Option<&str> {
        self.params.get(key).and_then(|values| values.first()).map(|s| s.as_str())
    }
    
    pub fn get_all(&self, key: &str) -> Option<&[String]> {
        self.params.get(key).map(|values| values.as_slice())
    }
    
    pub fn contains_key(&self, key: &str) -> bool {
        self.params.contains_key(key)
    }
    
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.params.keys()
    }
    
    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }
    
    pub fn len(&self) -> usize {
        self.params.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_empty_query() {
        let params = QueryParams::from_query_string("");
        assert!(params.is_empty());
        assert_eq!(params.len(), 0);
    }
    
    #[test]
    fn test_single_param() {
        let params = QueryParams::from_query_string("name=john");
        assert_eq!(params.get_first("name"), Some("john"));
        assert_eq!(params.len(), 1);
    }
    
    #[test]
    fn test_multiple_values() {
        let params = QueryParams::from_query_string("color=red&color=blue");
        assert_eq!(params.get_all("color"), Some(&["red".to_string(), "blue".to_string()][..]));
        assert_eq!(params.get_first("color"), Some("red"));
    }
    
    #[test]
    fn test_missing_key() {
        let params = QueryParams::from_query_string("name=john");
        assert_eq!(params.get_first("age"), None);
        assert!(!params.contains_key("age"));
    }
}