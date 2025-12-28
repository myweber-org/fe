use serde_json::{Value, Result};
use std::fs;

pub fn parse_json_file(file_path: &str) -> Result<Value> {
    let content = fs::read_to_string(file_path)?;
    let parsed: Value = serde_json::from_str(&content)?;
    Ok(parsed)
}

pub fn validate_json_structure(json: &Value, expected_keys: &[&str]) -> bool {
    if let Value::Object(map) = json {
        expected_keys.iter().all(|key| map.contains_key(*key))
    } else {
        false
    }
}

pub fn pretty_print_json(json: &Value) -> String {
    serde_json::to_string_pretty(json).unwrap_or_else(|_| String::from("Invalid JSON"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_valid_json() {
        let test_json = json!({
            "name": "test",
            "value": 42
        });
        let temp_file = "test_temp.json";
        fs::write(temp_file, test_json.to_string()).unwrap();
        
        let result = parse_json_file(temp_file);
        assert!(result.is_ok());
        
        fs::remove_file(temp_file).unwrap();
    }

    #[test]
    fn test_validate_structure() {
        let json = json!({
            "name": "test",
            "id": 1
        });
        
        assert!(validate_json_structure(&json, &["name", "id"]));
        assert!(!validate_json_structure(&json, &["name", "missing"]));
    }
}