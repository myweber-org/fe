use serde_json::{Value, Error as JsonError};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub enum LogError {
    IoError(std::io::Error),
    ParseError(JsonError),
    InvalidStructure(String),
}

impl From<std::io::Error> for LogError {
    fn from(err: std::io::Error) -> Self {
        LogError::IoError(err)
    }
}

impl From<JsonError> for LogError {
    fn from(err: JsonError) -> Self {
        LogError::ParseError(err)
    }
}

pub struct LogProcessor {
    pub total_lines: usize,
    pub valid_json_count: usize,
    pub error_count: usize,
}

impl LogProcessor {
    pub fn new() -> Self {
        LogProcessor {
            total_lines: 0,
            valid_json_count: 0,
            error_count: 0,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&mut self, path: P) -> Result<Vec<Value>, LogError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut results = Vec::new();

        for line_result in reader.lines() {
            self.total_lines += 1;
            let line = line_result?;

            match serde_json::from_str::<Value>(&line) {
                Ok(json_value) => {
                    self.valid_json_count += 1;
                    results.push(json_value);
                }
                Err(e) => {
                    self.error_count += 1;
                    eprintln!("Failed to parse line {}: {}", self.total_lines, e);
                }
            }
        }

        Ok(results)
    }

    pub fn extract_timestamps(&self, logs: &[Value]) -> Vec<String> {
        logs.iter()
            .filter_map(|log| log.get("timestamp").and_then(|ts| ts.as_str()))
            .map(|s| s.to_string())
            .collect()
    }

    pub fn filter_by_level(&self, logs: &[Value], level: &str) -> Vec<Value> {
        logs.iter()
            .filter(|log| {
                log.get("level")
                    .and_then(|lvl| lvl.as_str())
                    .map(|l| l == level)
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_valid_json() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{"timestamp": "2024-01-01T00:00:00Z", "level": "INFO", "message": "Test"}}"#).unwrap();
        writeln!(temp_file, r#"{{"timestamp": "2024-01-01T00:01:00Z", "level": "ERROR", "message": "Error"}}"#).unwrap();

        let mut processor = LogProcessor::new();
        let result = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(processor.total_lines, 2);
        assert_eq!(processor.valid_json_count, 2);
        assert_eq!(processor.error_count, 0);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_extract_timestamps() {
        let logs = vec![
            json!({"timestamp": "2024-01-01T00:00:00Z", "level": "INFO"}),
            json!({"timestamp": "2024-01-01T00:01:00Z", "level": "ERROR"}),
        ];

        let processor = LogProcessor::new();
        let timestamps = processor.extract_timestamps(&logs);

        assert_eq!(timestamps, vec!["2024-01-01T00:00:00Z", "2024-01-01T00:01:00Z"]);
    }

    #[test]
    fn test_filter_by_level() {
        let logs = vec![
            json!({"timestamp": "2024-01-01T00:00:00Z", "level": "INFO"}),
            json!({"timestamp": "2024-01-01T00:01:00Z", "level": "ERROR"}),
            json!({"timestamp": "2024-01-01T00:02:00Z", "level": "INFO"}),
        ];

        let processor = LogProcessor::new();
        let info_logs = processor.filter_by_level(&logs, "INFO");

        assert_eq!(info_logs.len(), 2);
        assert!(info_logs.iter().all(|log| log["level"] == "INFO"));
    }
}