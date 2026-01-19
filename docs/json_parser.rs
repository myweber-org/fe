use serde_json::{Value, json};
use std::fs;

pub fn parse_json_file(file_path: &str) -> Result<Value, String> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let json_value: Value = serde_json::from_str(&content)
        .map_err(|e| format!("Invalid JSON: {}", e))?;
    
    Ok(json_value)
}

pub fn validate_json_structure(value: &Value, expected_keys: &[&str]) -> bool {
    if let Value::Object(map) = value {
        expected_keys.iter().all(|key| map.contains_key(*key))
    } else {
        false
    }
}

pub fn pretty_print_json(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| String::from("Invalid JSON"))
}

pub fn create_sample_json() -> Value {
    json!({
        "name": "Data Processor",
        "version": "1.0.0",
        "active": true,
        "features": ["parsing", "validation", "formatting"],
        "metadata": {
            "author": "System",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_json_parsing() {
        let sample = create_sample_json();
        let pretty = pretty_print_json(&sample);
        assert!(pretty.contains("Data Processor"));
        
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), &pretty).unwrap();
        
        let parsed = parse_json_file(temp_file.path().to_str().unwrap()).unwrap();
        assert!(validate_json_structure(&parsed, &["name", "version", "active"]));
    }
}