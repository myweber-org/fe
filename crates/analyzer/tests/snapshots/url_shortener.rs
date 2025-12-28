
use std::collections::HashMap;
use std::sync::RwLock;
use lazy_static::lazy_static;
use nanoid::nanoid;
use url::Url;

lazy_static! {
    static ref STORAGE: RwLock<HashMap<String, String>> = RwLock::new(HashMap::new());
}

pub struct UrlShortener;

impl UrlShortener {
    pub fn shorten(original_url: &str) -> Result<String, String> {
        let parsed = Url::parse(original_url)
            .map_err(|_| "Invalid URL format".to_string())?;
        
        if parsed.scheme() != "http" && parsed.scheme() != "https" {
            return Err("Only HTTP/HTTPS URLs are supported".to_string());
        }

        let id = nanoid!(6);
        let short_url = format!("https://short.url/{}", id);
        
        STORAGE.write()
            .map_err(|_| "Storage lock error".to_string())?
            .insert(id.clone(), original_url.to_string());
        
        Ok(short_url)
    }

    pub fn resolve(short_url: &str) -> Option<String> {
        let id = short_url.trim_start_matches("https://short.url/");
        STORAGE.read()
            .ok()
            .and_then(|storage| storage.get(id).cloned())
    }

    pub fn stats() -> usize {
        STORAGE.read()
            .map(|storage| storage.len())
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_url_shortening() {
        let url = "https://example.com/page";
        let result = UrlShortener::shorten(url);
        assert!(result.is_ok());
        
        let short_url = result.unwrap();
        let resolved = UrlShortener::resolve(&short_url);
        assert_eq!(resolved, Some(url.to_string()));
    }

    #[test]
    fn test_invalid_url() {
        let result = UrlShortener::shorten("not-a-url");
        assert!(result.is_err());
    }

    #[test]
    fn test_stats() {
        let initial_count = UrlShortener::stats();
        let _ = UrlShortener::shorten("https://test.com");
        assert_eq!(UrlShortener::stats(), initial_count + 1);
    }
}