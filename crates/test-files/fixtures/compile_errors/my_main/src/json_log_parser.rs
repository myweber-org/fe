use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct LogParser {
    filters: HashMap<String, String>,
    extract_fields: Vec<String>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            filters: HashMap::new(),
            extract_fields: Vec::new(),
        }
    }

    pub fn add_filter(&mut self, key: &str, value: &str) {
        self.filters.insert(key.to_string(), value.to_string());
    }

    pub fn add_extract_field(&mut self, field: &str) {
        self.extract_fields.push(field.to_string());
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
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

    fn matches_filters(&self, json_value: &Value) -> bool {
        for (key, expected_value) in &self.filters {
            if let Some(actual_value) = json_value.get(key) {
                if actual_value.as_str() != Some(expected_value) {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    fn extract_fields(&self, json_value: &Value) -> Value {
        if self.extract_fields.is_empty() {
            return json_value.clone();
        }

        let mut result = serde_json::Map::new();
        for field in &self.extract_fields {
            if let Some(value) = json_value.get(field) {
                result.insert(field.clone(), value.clone());
            }
        }
        Value::Object(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parser_with_filters() {
        let mut parser = LogParser::new();
        parser.add_filter("level", "ERROR");
        parser.add_extract_field("timestamp");
        parser.add_extract_field("message");

        let test_data = json!({
            "timestamp": "2023-10-01T12:00:00Z",
            "level": "ERROR",
            "message": "Database connection failed",
            "service": "api"
        });

        assert!(parser.matches_filters(&test_data));
        
        let extracted = parser.extract_fields(&test_data);
        assert_eq!(extracted["timestamp"], "2023-10-01T12:00:00Z");
        assert_eq!(extracted["message"], "Database connection failed");
        assert!(extracted.get("service").is_none());
    }
}