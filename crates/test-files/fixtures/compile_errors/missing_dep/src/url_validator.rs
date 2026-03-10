use regex::Regex;
use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum ValidationError {
    InvalidFormat,
    UnsupportedProtocol,
    MissingHost,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidFormat => write!(f, "URL format is invalid"),
            ValidationError::UnsupportedProtocol => write!(f, "Protocol must be http or https"),
            ValidationError::MissingHost => write!(f, "URL must contain a host"),
        }
    }
}

impl Error for ValidationError {}

pub struct UrlValidator {
    pattern: Regex,
}

impl UrlValidator {
    pub fn new() -> Result<Self, regex::Error> {
        let pattern = Regex::new(
            r"^(https?)://([a-zA-Z0-9\-\.]+)(?::(\d+))?(?:/(.*))?$"
        )?;
        
        Ok(UrlValidator { pattern })
    }

    pub fn validate(&self, url: &str) -> Result<(), ValidationError> {
        let captures = match self.pattern.captures(url) {
            Some(caps) => caps,
            None => return Err(ValidationError::InvalidFormat),
        };

        let protocol = captures.get(1).map(|m| m.as_str());
        let host = captures.get(2).map(|m| m.as_str());

        match protocol {
            Some("http") | Some("https") => (),
            _ => return Err(ValidationError::UnsupportedProtocol),
        }

        if host.is_none() || host.unwrap().is_empty() {
            return Err(ValidationError::MissingHost);
        }

        Ok(())
    }

    pub fn extract_domain(&self, url: &str) -> Option<String> {
        self.pattern.captures(url)
            .and_then(|caps| caps.get(2))
            .map(|m| m.as_str().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_urls() {
        let validator = UrlValidator::new().unwrap();
        
        assert!(validator.validate("https://example.com").is_ok());
        assert!(validator.validate("http://localhost:8080").is_ok());
        assert!(validator.validate("https://sub.domain.co.uk/path").is_ok());
    }

    #[test]
    fn test_invalid_urls() {
        let validator = UrlValidator::new().unwrap();
        
        assert_eq!(validator.validate("ftp://example.com"), Err(ValidationError::UnsupportedProtocol));
        assert_eq!(validator.validate("https://"), Err(ValidationError::MissingHost));
        assert_eq!(validator.validate("://example.com"), Err(ValidationError::InvalidFormat));
    }

    #[test]
    fn test_domain_extraction() {
        let validator = UrlValidator::new().unwrap();
        
        assert_eq!(validator.extract_domain("https://github.com/rust-lang"), Some("github.com".to_string()));
        assert_eq!(validator.extract_domain("http://127.0.0.1:3000"), Some("127.0.0.1".to_string()));
        assert_eq!(validator.extract_domain("invalid-url"), None);
    }
}