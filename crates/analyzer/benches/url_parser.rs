
use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse(url: &str) -> Result<ParsedUrl, ParseError> {
        if url.trim().is_empty() {
            return Err(ParseError::EmptyUrl);
        }

        let parts: Vec<&str> = url.split('?').collect();
        let base = parts[0].to_string();
        
        let query_params = if parts.len() > 1 {
            Self::parse_query_string(parts[1])
        } else {
            HashMap::new()
        };

        Ok(ParsedUrl {
            base_url: base,
            query_params,
        })
    }

    fn parse_query_string(query: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        for pair in query.split('&') {
            let kv: Vec<&str> = pair.split('=').collect();
            if kv.len() == 2 {
                params.insert(
                    kv[0].to_string(),
                    kv[1].to_string(),
                );
            }
        }
        
        params
    }

    pub fn build_query_string(params: &HashMap<String, String>) -> String {
        let mut pairs: Vec<String> = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();
        
        pairs.sort();
        pairs.join("&")
    }
}

pub struct ParsedUrl {
    pub base_url: String,
    pub query_params: HashMap<String, String>,
}

impl ParsedUrl {
    pub fn to_string(&self) -> String {
        if self.query_params.is_empty() {
            self.base_url.clone()
        } else {
            format!(
                "{}?{}",
                self.base_url,
                UrlParser::build_query_string(&self.query_params)
            )
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    EmptyUrl,
    InvalidFormat,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_url() {
        let url = "https://example.com/path";
        let parsed = UrlParser::parse(url).unwrap();
        
        assert_eq!(parsed.base_url, "https://example.com/path");
        assert!(parsed.query_params.is_empty());
    }

    #[test]
    fn test_parse_url_with_query() {
        let url = "https://example.com/search?q=rust&sort=desc";
        let parsed = UrlParser::parse(url).unwrap();
        
        assert_eq!(parsed.base_url, "https://example.com/search");
        assert_eq!(parsed.query_params.get("q"), Some(&"rust".to_string()));
        assert_eq!(parsed.query_params.get("sort"), Some(&"desc".to_string()));
    }

    #[test]
    fn test_build_query_string() {
        let mut params = HashMap::new();
        params.insert("page".to_string(), "2".to_string());
        params.insert("limit".to_string(), "10".to_string());
        
        let query = UrlParser::build_query_string(&params);
        assert!(query == "limit=10&page=2" || query == "page=2&limit=10");
    }

    #[test]
    fn test_empty_url() {
        let result = UrlParser::parse("");
        assert_eq!(result, Err(ParseError::EmptyUrl));
    }
}