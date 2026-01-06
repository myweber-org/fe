
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ParseError {
    MalformedUrl,
    InvalidEncoding,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::MalformedUrl => write!(f, "URL format is invalid"),
            ParseError::InvalidEncoding => write!(f, "URL contains invalid percent encoding"),
        }
    }
}

impl Error for ParseError {}

pub fn parse_query_params(url: &str) -> Result<HashMap<String, String>, ParseError> {
    let query_start = url.find('?').ok_or(ParseError::MalformedUrl)?;
    let query_str = &url[query_start + 1..];
    
    let mut params = HashMap::new();
    
    for pair in query_str.split('&') {
        if pair.is_empty() {
            continue;
        }
        
        let mut parts = pair.splitn(2, '=');
        let key = parts.next().unwrap();
        let value = parts.next().unwrap_or("");
        
        let decoded_key = percent_decode(key).map_err(|_| ParseError::InvalidEncoding)?;
        let decoded_value = percent_decode(value).map_err(|_| ParseError::InvalidEncoding)?;
        
        params.insert(decoded_key, decoded_value);
    }
    
    Ok(params)
}

fn percent_decode(input: &str) -> Result<String, ()> {
    let mut result = Vec::new();
    let mut bytes = input.bytes();
    
    while let Some(byte) = bytes.next() {
        if byte == b'%' {
            let hex_high = bytes.next().ok_or(())?;
            let hex_low = bytes.next().ok_or(())?;
            
            let high = hex_to_nibble(hex_high).ok_or(())?;
            let low = hex_to_nibble(hex_low).ok_or(())?;
            
            result.push((high << 4) | low);
        } else if byte == b'+' {
            result.push(b' ');
        } else {
            result.push(byte);
        }
    }
    
    String::from_utf8(result).map_err(|_| ())
}

fn hex_to_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_parsing() {
        let url = "https://example.com/search?q=rust&lang=en&page=1";
        let params = parse_query_params(url).unwrap();
        
        assert_eq!(params.get("q"), Some(&"rust".to_string()));
        assert_eq!(params.get("lang"), Some(&"en".to_string()));
        assert_eq!(params.get("page"), Some(&"1".to_string()));
    }
    
    #[test]
    fn test_percent_decoding() {
        let url = "https://example.com/?query=hello%20world&special=%2B%26%3D";
        let params = parse_query_params(url).unwrap();
        
        assert_eq!(params.get("query"), Some(&"hello world".to_string()));
        assert_eq!(params.get("special"), Some(&"+&=".to_string()));
    }
    
    #[test]
    fn test_empty_value() {
        let url = "https://example.com/?flag=&empty";
        let params = parse_query_params(url).unwrap();
        
        assert_eq!(params.get("flag"), Some(&"".to_string()));
        assert_eq!(params.get("empty"), Some(&"".to_string()));
    }
    
    #[test]
    fn test_malformed_url() {
        let url = "https://example.com/path";
        let result = parse_query_params(url);
        
        assert!(matches!(result, Err(ParseError::MalformedUrl)));
    }
}