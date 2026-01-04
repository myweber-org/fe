
use std::collections::HashMap;
use std::sync::RwLock;
use lazy_static::lazy_static;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

lazy_static! {
    static ref URL_STORE: RwLock<HashMap<String, String>> = RwLock::new(HashMap::new());
}

pub fn shorten_url(original_url: &str) -> Result<String, &'static str> {
    if !original_url.starts_with("http://") && !original_url.starts_with("https://") {
        return Err("URL must start with http:// or https://");
    }

    let short_code: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect();

    let mut store = URL_STORE.write().unwrap();
    store.insert(short_code.clone(), original_url.to_string());

    Ok(short_code)
}

pub fn retrieve_url(short_code: &str) -> Option<String> {
    let store = URL_STORE.read().unwrap();
    store.get(short_code).cloned()
}

pub fn list_urls() -> Vec<(String, String)> {
    let store = URL_STORE.read().unwrap();
    store.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_url_shortening() {
        let url = "https://www.example.com";
        let result = shorten_url(url);
        assert!(result.is_ok());
        
        let short_code = result.unwrap();
        let retrieved = retrieve_url(&short_code);
        assert_eq!(retrieved, Some(url.to_string()));
    }

    #[test]
    fn test_invalid_url() {
        let url = "example.com";
        let result = shorten_url(url);
        assert!(result.is_err());
    }

    #[test]
    fn test_nonexistent_code() {
        let retrieved = retrieve_url("nonexistent");
        assert_eq!(retrieved, None);
    }
}