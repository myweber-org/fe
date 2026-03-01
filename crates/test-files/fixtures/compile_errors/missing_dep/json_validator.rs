
use serde_json::{Error, Value};
use std::fs;

pub fn validate_json_from_file(file_path: &str) -> Result<Value, Error> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| Error::io(e))?;
    serde_json::from_str(&content)
}

pub fn validate_json_string(json_str: &str) -> Result<Value, Error> {
    serde_json::from_str(json_str)
}

pub fn is_valid_json(json_str: &str) -> bool {
    validate_json_string(json_str).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_valid_json_string() {
        let valid_json = r#"{"name": "test", "value": 42}"#;
        assert!(is_valid_json(valid_json));
    }

    #[test]
    fn test_invalid_json_string() {
        let invalid_json = r#"{"name": test, "value": 42}"#;
        assert!(!is_valid_json(invalid_json));
    }

    #[test]
    fn test_parse_valid_json() {
        let valid_json = r#"{"key": "value"}"#;
        let result = validate_json_string(valid_json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), json!({"key": "value"}));
    }
}