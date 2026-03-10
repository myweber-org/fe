
use std::collections::HashMap;

pub struct UrlParser;

impl UrlParser {
    pub fn parse_query_string(url: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if let Some(query_start) = url.find('?') {
            let query_string = &url[query_start + 1..];
            
            for param in query_string.split('&') {
                if let Some(equal_pos) = param.find('=') {
                    let key = &param[..equal_pos];
                    let value = &param[equal_pos + 1..];
                    
                    if !key.is_empty() {
                        params.insert(
                            key.to_string(),
                            percent_decode(value).unwrap_or_else(|| value.to_string())
                        );
                    }
                }
            }
        }
        
        params
    }
    
    pub fn extract_domain(url: &str) -> Option<String> {
        let url_lower = url.to_lowercase();
        
        if url_lower.starts_with("http://") || url_lower.starts_with("https://") {
            if let Some(slash_pos) = url[8..].find('/') {
                return Some(url[..8 + slash_pos].to_string());
            }
            return Some(url.to_string());
        }
        
        None
    }
}

fn percent_decode(input: &str) -> Option<String> {
    let mut result = String::new();
    let mut chars = input.chars().collect::<Vec<_>>();
    let mut i = 0;
    
    while i < chars.len() {
        if chars[i] == '%' && i + 2 < chars.len() {
            if let (Some(h1), Some(h2)) = (chars[i+1].to_digit(16), chars[i+2].to_digit(16)) {
                let byte = (h1 << 4) | h2;
                result.push(byte as u8 as char);
                i += 3;
                continue;
            }
        }
        result.push(chars[i]);
        i += 1;
    }
    
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_query_parsing() {
        let url = "https://example.com/search?q=rust&lang=en&page=2";
        let params = UrlParser::parse_query_string(url);
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("lang"), Some(&"en".to_string()));
        assert_eq!(params.get("page"), Some(&"2".to_string()));
    }
    
    #[test]
    fn test_domain_extraction() {
        let url = "https://api.github.com/users/rust-lang";
        let domain = UrlParser::extract_domain(url);
        
        assert_eq!(domain, Some("https://api.github.com".to_string()));
    }
    
    #[test]
    fn test_percent_decoding() {
        let encoded = "hello%20world%21";
        let decoded = percent_decode(encoded).unwrap();
        
        assert_eq!(decoded, "hello world!");
    }
}