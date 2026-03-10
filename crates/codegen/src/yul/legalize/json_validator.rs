use serde_json::{Value, Error as JsonError};
use std::fs;
use std::path::Path;

pub fn validate_json_file(file_path: &str) -> Result<Value, JsonError> {
    let path = Path::new(file_path);
    let content = fs::read_to_string(path)
        .map_err(|e| JsonError::io(e))?;
    
    serde_json::from_str(&content)
}

pub fn validate_json_string(json_str: &str) -> Result<Value, JsonError> {
    serde_json::from_str(json_str)
}

pub fn is_valid_json(json_str: &str) -> bool {
    validate_json_string(json_str).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_json_string() {
        let valid_json = r#"{"name": "test", "value": 42}"#;
        assert!(is_valid_json(valid_json));
    }

    #[test]
    fn test_invalid_json_string() {
        let invalid_json = r#"{"name": "test", "value": }"#;
        assert!(!is_valid_json(invalid_json));
    }

    #[test]
    fn test_parse_valid_json() {
        let json_str = r#"{"temperature": 25.5, "active": true}"#;
        let result = validate_json_string(json_str);
        assert!(result.is_ok());
        
        let parsed = result.unwrap();
        assert_eq!(parsed["temperature"].as_f64(), Some(25.5));
        assert_eq!(parsed["active"].as_bool(), Some(true));
    }
}