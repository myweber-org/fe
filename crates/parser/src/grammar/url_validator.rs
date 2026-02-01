use regex::Regex;

pub fn is_valid_url(url: &str) -> bool {
    let pattern = r"^https?://(?:[-\w.]|(?:%[\da-fA-F]{2}))+(?::\d+)?(?:/[-\w@:%._\+~#=]*)*(?:\?[-\w@:%._\+~#=]*)?(?:#[-\w@:%._\+~#=]*)?$";
    
    let re = match Regex::new(pattern) {
        Ok(regex) => regex,
        Err(_) => return false,
    };
    
    re.is_match(url)
}

pub fn extract_domain(url: &str) -> Option<String> {
    if !is_valid_url(url) {
        return None;
    }
    
    let domain_pattern = r"^https?://([^/:]+)";
    let re = match Regex::new(domain_pattern) {
        Ok(regex) => regex,
        Err(_) => return None,
    };
    
    re.captures(url)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_urls() {
        assert!(is_valid_url("https://example.com"));
        assert!(is_valid_url("http://sub.example.com/path"));
        assert!(is_valid_url("https://example.com:8080/api?query=test"));
    }

    #[test]
    fn test_invalid_urls() {
        assert!(!is_valid_url("example.com"));
        assert!(!is_valid_url("ftp://example.com"));
        assert!(!is_valid_url("https://"));
    }

    #[test]
    fn test_domain_extraction() {
        assert_eq!(extract_domain("https://example.com"), Some("example.com".to_string()));
        assert_eq!(extract_domain("http://sub.example.com/path"), Some("sub.example.com".to_string()));
        assert_eq!(extract_domain("invalid-url"), None);
    }
}