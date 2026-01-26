use url::Url;

pub struct UrlParser;

impl UrlParser {
    pub fn parse(url_str: &str) -> Result<ParsedUrl, String> {
        let url = Url::parse(url_str).map_err(|e| e.to_string())?;
        
        let domain = url.host_str()
            .map(|h| h.to_string())
            .ok_or_else(|| "No domain found".to_string())?;
        
        let mut query_params = std::collections::HashMap::new();
        for (key, value) in url.query_pairs() {
            query_params.insert(key.into_owned(), value.into_owned());
        }
        
        Ok(ParsedUrl {
            domain,
            query_params,
            full_url: url.to_string(),
        })
    }
}

pub struct ParsedUrl {
    pub domain: String,
    pub query_params: std::collections::HashMap<String, String>,
    pub full_url: String,
}

impl ParsedUrl {
    pub fn get_param(&self, key: &str) -> Option<&String> {
        self.query_params.get(key)
    }
    
    pub fn has_param(&self, key: &str) -> bool {
        self.query_params.contains_key(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_valid_url() {
        let url = "https://example.com/search?q=rust&lang=en";
        let parsed = UrlParser::parse(url).unwrap();
        
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.get_param("q"), Some(&"rust".to_string()));
        assert_eq!(parsed.get_param("lang"), Some(&"en".to_string()));
        assert!(parsed.has_param("q"));
        assert!(!parsed.has_param("nonexistent"));
    }
    
    #[test]
    fn test_parse_invalid_url() {
        let url = "not-a-valid-url";
        let result = UrlParser::parse(url);
        assert!(result.is_err());
    }
}