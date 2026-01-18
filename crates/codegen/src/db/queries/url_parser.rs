use std::collections::HashMap;

pub struct QueryParser;

impl QueryParser {
    pub fn parse(query_string: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if query_string.is_empty() {
            return params;
        }

        for pair in query_string.split('&') {
            let mut parts = pair.splitn(2, '=');
            if let Some(key) = parts.next() {
                let value = parts.next().unwrap_or("");
                params.insert(
                    key.to_string(),
                    urlencoding::decode(value)
                        .unwrap_or_else(|_| value.into())
                        .to_string(),
                );
            }
        }
        
        params
    }

    pub fn parse_from_url(url: &str) -> Option<HashMap<String, String>> {
        url.split('?')
            .nth(1)
            .map(|query| Self::parse(query))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_query() {
        let query = "name=john&age=25";
        let params = QueryParser::parse(query);
        
        assert_eq!(params.get("name"), Some(&"john".to_string()));
        assert_eq!(params.get("age"), Some(&"25".to_string()));
    }

    #[test]
    fn test_parse_encoded_values() {
        let query = "city=New%20York&country=USA";
        let params = QueryParser::parse(query);
        
        assert_eq!(params.get("city"), Some(&"New York".to_string()));
        assert_eq!(params.get("country"), Some(&"USA".to_string()));
    }

    #[test]
    fn test_parse_empty_query() {
        let params = QueryParser::parse("");
        assert!(params.is_empty());
    }

    #[test]
    fn test_parse_from_url() {
        let url = "https://example.com/search?q=rust&lang=en";
        let params = QueryParser::parse_from_url(url).unwrap();
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("lang"), Some(&"en".to_string()));
    }
}