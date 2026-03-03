use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct LogParser {
    filters: HashMap<String, String>,
    required_fields: Vec<String>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            filters: HashMap::new(),
            required_fields: Vec::new(),
        }
    }

    pub fn add_filter(&mut self, key: &str, value: &str) {
        self.filters.insert(key.to_string(), value.to_string());
    }

    pub fn require_field(&mut self, field: &str) {
        self.required_fields.push(field.to_string());
    }

    pub fn parse_file(&self, path: &str) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut results = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(json_value) = serde_json::from_str::<Value>(&line) {
                if self.matches_filters(&json_value) {
                    let extracted = self.extract_fields(&json_value);
                    results.push(extracted);
                }
            }
        }

        Ok(results)
    }

    fn matches_filters(&self, json: &Value) -> bool {
        for (key, expected_value) in &self.filters {
            if let Some(actual_value) = json.get(key) {
                if actual_value.as_str() != Some(expected_value) {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    fn extract_fields(&self, json: &Value) -> Value {
        if self.required_fields.is_empty() {
            return json.clone();
        }

        let mut result_map = serde_json::Map::new();
        for field in &self.required_fields {
            if let Some(value) = json.get(field) {
                result_map.insert(field.clone(), value.clone());
            }
        }
        Value::Object(result_map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_with_filters() {
        let mut parser = LogParser::new();
        parser.add_filter("level", "ERROR");
        parser.require_field("timestamp");
        parser.require_field("message");

        let test_log = r#"{"level": "ERROR", "timestamp": "2023-10-01T12:00:00Z", "message": "Something failed"}"#;
        let json_value: Value = serde_json::from_str(test_log).unwrap();
        
        assert!(parser.matches_filters(&json_value));
        
        let extracted = parser.extract_fields(&json_value);
        assert!(extracted.get("timestamp").is_some());
        assert!(extracted.get("message").is_some());
        assert!(extracted.get("level").is_none());
    }
}