use std::collections::HashMap;
use std::hash::{Hash, Hasher};

struct UrlShortener {
    urls: HashMap<String, String>,
    counter: u64,
}

impl UrlShortener {
    fn new() -> Self {
        UrlShortener {
            urls: HashMap::new(),
            counter: 0,
        }
    }

    fn shorten(&mut self, url: &str) -> String {
        let key = self.generate_key();
        self.urls.insert(key.clone(), url.to_string());
        format!("https://short.url/{}", key)
    }

    fn expand(&self, short_url: &str) -> Option<&String> {
        let key = short_url.split('/').last().unwrap();
        self.urls.get(key)
    }

    fn generate_key(&mut self) -> String {
        self.counter += 1;
        base62_encode(self.counter)
    }
}

fn base62_encode(mut num: u64) -> String {
    const CHARS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    let mut result = String::new();
    
    while num > 0 {
        let remainder = (num % 62) as usize;
        result.push(CHARS[remainder] as char);
        num /= 62;
    }
    
    result.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_shortener() {
        let mut shortener = UrlShortener::new();
        let original = "https://www.example.com/very/long/url/path";
        let short = shortener.shorten(original);
        assert_eq!(shortener.expand(&short), Some(&original.to_string()));
    }

    #[test]
    fn test_base62_encoding() {
        assert_eq!(base62_encode(0), "");
        assert_eq!(base62_encode(1), "1");
        assert_eq!(base62_encode(61), "z");
        assert_eq!(base62_encode(62), "10");
        assert_eq!(base62_encode(3844), "100");
    }
}