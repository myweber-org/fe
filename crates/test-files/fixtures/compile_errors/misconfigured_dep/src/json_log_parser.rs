
use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LogParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Missing required field: {0}")]
    MissingField(String),
}

pub struct JsonLogParser {
    file_path: String,
}

impl JsonLogParser {
    pub fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
        }
    }

    pub fn parse(&self) -> Result<Vec<Value>, LogParseError> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut logs = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line_content = line?;
            if line_content.trim().is_empty() {
                continue;
            }

            let json_value: Value = serde_json::from_str(&line_content)?;
            
            if !json_value.is_object() {
                return Err(LogParseError::MissingField(
                    format!("Line {}: Expected JSON object", line_num + 1)
                ));
            }

            logs.push(json_value);
        }

        Ok(logs)
    }

    pub fn filter_by_level(&self, level: &str) -> Result<Vec<Value>, LogParseError> {
        let logs = self.parse()?;
        let filtered: Vec<Value> = logs
            .into_iter()
            .filter(|log| {
                log.get("level")
                    .and_then(|v| v.as_str())
                    .map(|l| l.eq_ignore_ascii_case(level))
                    .unwrap_or(false)
            })
            .collect();

        Ok(filtered)
    }

    pub fn extract_timestamps(&self) -> Result<Vec<String>, LogParseError> {
        let logs = self.parse()?;
        let timestamps: Vec<String> = logs
            .iter()
            .filter_map(|log| {
                log.get("timestamp")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .collect();

        if timestamps.is_empty() {
            return Err(LogParseError::MissingField(
                "No timestamps found in logs".to_string()
            ));
        }

        Ok(timestamps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_logs() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"{{"timestamp": "2024-01-15T10:30:00Z", "level": "INFO", "message": "System started"}}"#).unwrap();
        writeln!(file, r#"{{"timestamp": "2024-01-15T10:31:00Z", "level": "ERROR", "message": "Connection failed"}}"#).unwrap();
        writeln!(file, r#"{{"timestamp": "2024-01-15T10:32:00Z", "level": "WARN", "message": "High memory usage"}}"#).unwrap();
        file
    }

    #[test]
    fn test_parse_valid_logs() {
        let test_file = create_test_logs();
        let parser = JsonLogParser::new(test_file.path().to_str().unwrap());
        let result = parser.parse();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 3);
    }

    #[test]
    fn test_filter_by_level() {
        let test_file = create_test_logs();
        let parser = JsonLogParser::new(test_file.path().to_str().unwrap());
        let errors = parser.filter_by_level("ERROR").unwrap();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0]["level"], "ERROR");
    }

    #[test]
    fn test_extract_timestamps() {
        let test_file = create_test_logs();
        let parser = JsonLogParser::new(test_file.path().to_str().unwrap());
        let timestamps = parser.extract_timestamps().unwrap();
        assert_eq!(timestamps.len(), 3);
        assert!(timestamps[0].contains("2024-01-15"));
    }
}