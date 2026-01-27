use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct LogProcessor {
    min_severity: u8,
    filter_key: Option<String>,
    filter_value: Option<String>,
}

impl LogProcessor {
    pub fn new(min_severity: u8) -> Self {
        LogProcessor {
            min_severity,
            filter_key: None,
            filter_value: None,
        }
    }

    pub fn with_filter(mut self, key: &str, value: &str) -> Self {
        self.filter_key = Some(key.to_string());
        self.filter_value = Some(value.to_string());
        self
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut results = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(json_value) = serde_json::from_str::<Value>(&line) {
                if self.matches_criteria(&json_value) {
                    results.push(line);
                }
            }
        }

        Ok(results)
    }

    fn matches_criteria(&self, json: &Value) -> bool {
        if let Some(severity) = json.get("severity").and_then(|v| v.as_u64()) {
            if (severity as u8) < self.min_severity {
                return false;
            }
        }

        if let (Some(ref key), Some(ref value)) = (&self.filter_key, &self.filter_value) {
            if let Some(target_value) = json.get(key).and_then(|v| v.as_str()) {
                return target_value.contains(value);
            }
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_filtering() {
        let logs = r#"{"severity": 3, "message": "System started", "component": "boot"}
{"severity": 1, "message": "Low priority event", "component": "monitor"}
{"severity": 5, "message": "Critical error", "component": "database"}"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", logs).unwrap();

        let processor = LogProcessor::new(3);
        let results = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(results.len(), 2);
        assert!(results[0].contains("System started"));
        assert!(results[1].contains("Critical error"));
    }

    #[test]
    fn test_key_value_filter() {
        let logs = r#"{"severity": 3, "component": "api", "user": "alice"}
{"severity": 3, "component": "api", "user": "bob"}
{"severity": 3, "component": "worker", "user": "alice"}"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", logs).unwrap();

        let processor = LogProcessor::new(1).with_filter("component", "api");
        let results = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(results.len(), 2);
        assert!(results[0].contains("api"));
        assert!(results[1].contains("api"));
    }
}