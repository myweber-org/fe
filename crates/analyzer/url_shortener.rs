
use std::collections::HashMap;
use std::sync::RwLock;
use lazy_static::lazy_static;
use rand::{distributions::Alphanumeric, Rng};
use url::Url;

lazy_static! {
    static ref URL_STORE: RwLock<HashMap<String, String>> = RwLock::new(HashMap::new());
}

const SHORT_CODE_LENGTH: usize = 7;
const BASE_URL: &str = "https://short.url/";

pub fn shorten_url(original_url: &str) -> Result<String, String> {
    if !is_valid_url(original_url) {
        return Err("Invalid URL format".to_string());
    }

    let short_code = generate_short_code();
    let shortened = format!("{}{}", BASE_URL, short_code);

    {
        let mut store = URL_STORE.write().unwrap();
        store.insert(short_code.clone(), original_url.to_string());
    }

    Ok(shortened)
}

pub fn retrieve_url(short_code: &str) -> Option<String> {
    let store = URL_STORE.read().unwrap();
    store.get(short_code).cloned()
}

fn generate_short_code() -> String {
    let mut rng = rand::thread_rng();
    (0..SHORT_CODE_LENGTH)
        .map(|_| rng.sample(Alphanumeric) as char)
        .collect()
}

fn is_valid_url(url_str: &str) -> bool {
    Url::parse(url_str).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_validation() {
        assert!(is_valid_url("https://example.com"));
        assert!(!is_valid_url("not-a-url"));
    }

    #[test]
    fn test_shorten_and_retrieve() {
        let original = "https://www.rust-lang.org";
        let shortened = shorten_url(original).unwrap();
        
        let short_code = shortened.trim_start_matches(BASE_URL);
        let retrieved = retrieve_url(short_code).unwrap();
        
        assert_eq!(retrieved, original);
    }

    #[test]
    fn test_invalid_url() {
        let result = shorten_url("invalid-url");
        assert!(result.is_err());
    }
}