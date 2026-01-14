
use regex::Regex;

pub fn parse_url(url: &str) -> Option<String> {
    let pattern = r"^(https?://)?([\w-]+\.)+[\w-]+(:\d+)?(/[\w-./?%&=]*)?$";
    let re = Regex::new(pattern).unwrap();
    
    if re.is_match(url) {
        Some(url.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_urls() {
        assert!(parse_url("https://example.com").is_some());
        assert!(parse_url("http://sub.domain.co.uk/path").is_some());
        assert!(parse_url("localhost:8080/api").is_some());
    }

    #[test]
    fn test_invalid_urls() {
        assert!(parse_url("not-a-url").is_none());
        assert!(parse_url("http://").is_none());
        assert!(parse_url("://example.com").is_none());
    }
}