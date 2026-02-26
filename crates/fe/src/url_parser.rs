
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, PartialEq)]
pub struct ParsedUrl {
    pub scheme: String,
    pub domain: String,
    pub path: String,
    pub query_params: HashMap<String, String>,
    pub fragment: Option<String>,
}

impl ParsedUrl {
    pub fn new(url: &str) -> Result<Self, Box<dyn Error>> {
        let mut scheme = String::new();
        let mut domain = String::new();
        let mut path = String::new();
        let mut query_params = HashMap::new();
        let mut fragment = None;

        let parts: Vec<&str> = url.split("://").collect();
        if parts.len() != 2 {
            return Err("Invalid URL format".into());
        }

        scheme = parts[0].to_string();
        let rest = parts[1];

        let hash_split: Vec<&str> = rest.split('#').collect();
        let before_fragment = hash_split[0];
        if hash_split.len() > 1 {
            fragment = Some(hash_split[1].to_string());
        }

        let query_split: Vec<&str> = before_fragment.split('?').collect();
        let authority_path = query_split[0];
        if query_split.len() > 1 {
            for param in query_split[1].split('&') {
                let pair: Vec<&str> = param.split('=').collect();
                if pair.len() == 2 {
                    query_params.insert(pair[0].to_string(), pair[1].to_string());
                }
            }
        }

        let path_split: Vec<&str> = authority_path.splitn(2, '/').collect();
        domain = path_split[0].to_string();
        if path_split.len() > 1 {
            path = format!("/{}", path_split[1]);
        }

        Ok(ParsedUrl {
            scheme,
            domain,
            path,
            query_params,
            fragment,
        })
    }

    pub fn get_root_domain(&self) -> Option<String> {
        let parts: Vec<&str> = self.domain.split('.').collect();
        if parts.len() >= 2 {
            let last_two = &parts[parts.len() - 2..];
            Some(last_two.join("."))
        } else {
            None
        }
    }

    pub fn get_query_param(&self, key: &str) -> Option<&String> {
        self.query_params.get(key)
    }
}

pub fn extract_urls(text: &str) -> Vec<String> {
    let mut urls = Vec::new();
    let words: Vec<&str> = text.split_whitespace().collect();
    
    for word in words {
        if word.starts_with("http://") || word.starts_with("https://") {
            urls.push(word.to_string());
        }
    }
    
    urls
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_url() {
        let url = "https://example.com/path/to/resource";
        let parsed = ParsedUrl::new(url).unwrap();
        
        assert_eq!(parsed.scheme, "https");
        assert_eq!(parsed.domain, "example.com");
        assert_eq!(parsed.path, "/path/to/resource");
        assert!(parsed.query_params.is_empty());
        assert_eq!(parsed.fragment, None);
    }

    #[test]
    fn test_parse_url_with_query() {
        let url = "https://api.example.com/search?q=rust&limit=10&sort=desc";
        let parsed = ParsedUrl::new(url).unwrap();
        
        assert_eq!(parsed.scheme, "https");
        assert_eq!(parsed.domain, "api.example.com");
        assert_eq!(parsed.path, "/search");
        assert_eq!(parsed.get_query_param("q"), Some(&"rust".to_string()));
        assert_eq!(parsed.get_query_param("limit"), Some(&"10".to_string()));
        assert_eq!(parsed.get_query_param("sort"), Some(&"desc".to_string()));
    }

    #[test]
    fn test_root_domain_extraction() {
        let url = "https://subdomain.api.example.co.uk/path";
        let parsed = ParsedUrl::new(url).unwrap();
        
        assert_eq!(parsed.get_root_domain(), Some("example.co.uk".to_string()));
    }

    #[test]
    fn test_extract_urls_from_text() {
        let text = "Visit https://example.com and https://api.test.org/data?key=value for more info";
        let urls = extract_urls(text);
        
        assert_eq!(urls.len(), 2);
        assert_eq!(urls[0], "https://example.com");
        assert_eq!(urls[1], "https://api.test.org/data?key=value");
    }
}