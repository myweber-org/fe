use regex::Regex;

pub fn is_valid_url(url: &str) -> bool {
    let pattern = r"^https?://(?:[-\w]+\.)+[-\w]{2,}(?:/[\w\-./?%&=]*)?$";
    let re = Regex::new(pattern).unwrap();
    re.is_match(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_urls() {
        assert!(is_valid_url("https://example.com"));
        assert!(is_valid_url("http://sub.example.com/path"));
        assert!(is_valid_url("https://api.service.io/v1/resource?id=123"));
    }

    #[test]
    fn test_invalid_urls() {
        assert!(!is_valid_url("example.com"));
        assert!(!is_valid_url("ftp://example.com"));
        assert!(!is_valid_url("https://"));
        assert!(!is_valid_url("://example.com"));
    }
}