use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct LogParser {
    filters: HashMap<String, String>,
    fields_to_extract: Vec<String>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            filters: HashMap::new(),
            fields_to_extract: Vec::new(),
        }
    }

    pub fn add_filter(&mut self, key: &str, value: &str) {
        self.filters.insert(key.to_string(), value.to_string());
    }

    pub fn add_field_to_extract(&mut self, field: &str) {
        self.fields_to_extract.push(field.to_string());
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<HashMap<String, Value>>, Box<dyn std::error::Error>> {
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

    fn extract_fields(&self, json: &Value) -> HashMap<String, Value> {
        let mut result = HashMap::new();
        for field in &self.fields_to_extract {
            if let Some(value) = json.get(field) {
                result.insert(field.clone(), value.clone());
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parser_with_filters() {
        let mut parser = LogParser::new();
        parser.add_filter("level", "error");
        parser.add_field_to_extract("timestamp");
        parser.add_field_to_extract("message");

        let test_data = r#"{"level": "error", "timestamp": "2023-10-01T12:00:00Z", "message": "System failure"}"#;
        let json_value: Value = serde_json::from_str(test_data).unwrap();

        assert!(parser.matches_filters(&json_value));
        
        let extracted = parser.extract_fields(&json_value);
        assert_eq!(extracted.len(), 2);
        assert!(extracted.contains_key("timestamp"));
        assert!(extracted.contains_key("message"));
    }
}