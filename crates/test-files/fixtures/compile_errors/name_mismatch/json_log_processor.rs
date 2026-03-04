use serde_json::{Value, Error as JsonError};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub metadata: Value,
}

#[derive(Debug)]
pub enum LogError {
    IoError(std::io::Error),
    JsonError(JsonError),
    InvalidFormat(String),
}

impl From<std::io::Error> for LogError {
    fn from(err: std::io::Error) -> Self {
        LogError::IoError(err)
    }
}

impl From<JsonError> for LogError {
    fn from(err: JsonError) -> Self {
        LogError::JsonError(err)
    }
}

pub struct LogProcessor {
    pub entries: Vec<LogEntry>,
    pub error_count: usize,
}

impl LogProcessor {
    pub fn new() -> Self {
        LogProcessor {
            entries: Vec::new(),
            error_count: 0,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), LogError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            match self.parse_line(&line) {
                Ok(entry) => self.entries.push(entry),
                Err(e) => {
                    eprintln!("Error parsing line {}: {:?}", line_num + 1, e);
                    self.error_count += 1;
                }
            }
        }

        Ok(())
    }

    fn parse_line(&self, line: &str) -> Result<LogEntry, LogError> {
        let json_value: Value = serde_json::from_str(line)?;

        let timestamp = json_value["timestamp"]
            .as_str()
            .ok_or_else(|| LogError::InvalidFormat("Missing timestamp".to_string()))?
            .to_string();

        let level = json_value["level"]
            .as_str()
            .ok_or_else(|| LogError::InvalidFormat("Missing level".to_string()))?
            .to_string();

        let message = json_value["message"]
            .as_str()
            .ok_or_else(|| LogError::InvalidFormat("Missing message".to_string()))?
            .to_string();

        let metadata = json_value["metadata"].clone();

        Ok(LogEntry {
            timestamp,
            level,
            message,
            metadata,
        })
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn count_by_level(&self) -> std::collections::HashMap<String, usize> {
        let mut counts = std::collections::HashMap::new();
        for entry in &self.entries {
            *counts.entry(entry.level.clone()).or_insert(0) += 1;
        }
        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_valid_log() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let log_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","message":"Service started","metadata":{"pid":1234}}
{"timestamp":"2024-01-15T10:31:00Z","level":"ERROR","message":"Connection failed","metadata":{"retry_count":3}}"#;
        
        write!(temp_file, "{}", log_data).unwrap();
        
        let mut processor = LogProcessor::new();
        processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(processor.entries.len(), 2);
        assert_eq!(processor.error_count, 0);
        
        let error_entries = processor.filter_by_level("ERROR");
        assert_eq!(error_entries.len(), 1);
        assert_eq!(error_entries[0].message, "Connection failed");
        
        let counts = processor.count_by_level();
        assert_eq!(counts.get("INFO"), Some(&1));
        assert_eq!(counts.get("ERROR"), Some(&1));
    }

    #[test]
    fn test_process_invalid_json() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let log_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","message":"Valid entry"}
Invalid JSON here
{"timestamp":"2024-01-15T10:31:00Z","level":"WARN","message":"Another valid"}"#;
        
        write!(temp_file, "{}", log_data).unwrap();
        
        let mut processor = LogProcessor::new();
        processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(processor.entries.len(), 2);
        assert_eq!(processor.error_count, 1);
    }
}