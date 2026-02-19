use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum LogLevel {
    ERROR,
    WARN,
    INFO,
    DEBUG,
    TRACE,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub service: String,
    pub message: String,
    pub metadata: Option<serde_json::Value>,
}

pub struct LogParser {
    file_path: String,
}

impl LogParser {
    pub fn new(file_path: &str) -> Self {
        LogParser {
            file_path: file_path.to_string(),
        }
    }

    pub fn parse(&self) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<LogEntry>(&line) {
                Ok(entry) => entries.push(entry),
                Err(e) => eprintln!("Failed to parse line: {} - Error: {}", line, e),
            }
        }

        Ok(entries)
    }

    pub fn filter_by_level(&self, level: LogLevel) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let entries = self.parse()?;
        let filtered: Vec<LogEntry> = entries
            .into_iter()
            .filter(|entry| entry.level == level)
            .collect();
        Ok(filtered)
    }

    pub fn count_by_service(&self) -> Result<std::collections::HashMap<String, usize>, Box<dyn std::error::Error>> {
        let entries = self.parse()?;
        let mut counts = std::collections::HashMap::new();

        for entry in entries {
            *counts.entry(entry.service).or_insert(0) += 1;
        }

        Ok(counts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_logs() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let log_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"ERROR","service":"auth","message":"Authentication failed","metadata":{"user_id":123}}
{"timestamp":"2024-01-15T10:31:00Z","level":"INFO","service":"api","message":"Request processed","metadata":null}"#;
        write!(temp_file, "{}", log_data).unwrap();

        let parser = LogParser::new(temp_file.path().to_str().unwrap());
        let result = parser.parse();
        assert!(result.is_ok());
        let entries = result.unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].level, LogLevel::ERROR);
        assert_eq!(entries[1].service, "api");
    }

    #[test]
    fn test_filter_by_level() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let log_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"ERROR","service":"auth","message":"Authentication failed","metadata":null}
{"timestamp":"2024-01-15T10:31:00Z","level":"INFO","service":"api","message":"Request processed","metadata":null}
{"timestamp":"2024-01-15T10:32:00Z","level":"ERROR","service":"db","message":"Connection timeout","metadata":null}"#;
        write!(temp_file, "{}", log_data).unwrap();

        let parser = LogParser::new(temp_file.path().to_str().unwrap());
        let errors = parser.filter_by_level(LogLevel::ERROR).unwrap();
        assert_eq!(errors.len(), 2);
        assert!(errors.iter().all(|e| e.level == LogLevel::ERROR));
    }

    #[test]
    fn test_count_by_service() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let log_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"ERROR","service":"auth","message":"Authentication failed","metadata":null}
{"timestamp":"2024-01-15T10:31:00Z","level":"INFO","service":"api","message":"Request processed","metadata":null}
{"timestamp":"2024-01-15T10:32:00Z","level":"ERROR","service":"auth","message":"Invalid token","metadata":null}"#;
        write!(temp_file, "{}", log_data).unwrap();

        let parser = LogParser::new(temp_file.path().to_str().unwrap());
        let counts = parser.count_by_service().unwrap();
        assert_eq!(counts.get("auth"), Some(&2));
        assert_eq!(counts.get("api"), Some(&1));
        assert_eq!(counts.get("unknown"), None);
    }
}