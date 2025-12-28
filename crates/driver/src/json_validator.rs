use serde_json::{Error, Value};

pub fn is_valid_json(json_str: &str) -> bool {
    serde_json::from_str::<Value>(json_str).is_ok()
}

pub fn parse_json(json_str: &str) -> Result<Value, Error> {
    serde_json::from_str(json_str)
}

pub fn validate_json_structure(json_str: &str, expected_keys: &[&str]) -> bool {
    if let Ok(parsed) = parse_json(json_str) {
        if let Value::Object(map) = parsed {
            return expected_keys.iter().all(|key| map.contains_key(*key));
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_json() {
        let valid_json = r#"{"name": "test", "value": 42}"#;
        assert!(is_valid_json(valid_json));
    }

    #[test]
    fn test_invalid_json() {
        let invalid_json = r#"{"name": test, "value": 42}"#;
        assert!(!is_valid_json(invalid_json));
    }

    #[test]
    fn test_validate_structure() {
        let json = r#"{"id": 1, "title": "example"}"#;
        assert!(validate_json_structure(json, &["id", "title"]));
        assert!(!validate_json_structure(json, &["id", "missing"]));
    }
}