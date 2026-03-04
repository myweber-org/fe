use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum LogLevel {
    INFO,
    WARN,
    ERROR,
    DEBUG,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub message: String,
    pub component: String,
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

    pub fn parse(&self) -> Result<Vec<LogEntry>, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let entry: LogEntry = serde_json::from_str(&line)?;
            entries.push(entry);
        }

        Ok(entries)
    }

    pub fn filter_by_level(&self, level: LogLevel) -> Result<Vec<LogEntry>, Box<dyn Error>> {
        let entries = self.parse()?;
        let filtered: Vec<LogEntry> = entries
            .into_iter()
            .filter(|entry| entry.level == level)
            .collect();
        Ok(filtered)
    }

    pub fn count_entries(&self) -> Result<usize, Box<dyn Error>> {
        let entries = self.parse()?;
        Ok(entries.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_log_entries() {
        let log_data = r#"{"timestamp":"2023-10-01T12:00:00Z","level":"INFO","message":"System started","component":"core"}
{"timestamp":"2023-10-01T12:01:00Z","level":"ERROR","message":"Connection failed","component":"network"}"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", log_data).unwrap();

        let parser = LogParser::new(temp_file.path().to_str().unwrap());
        let entries = parser.parse().unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].level, LogLevel::INFO);
        assert_eq!(entries[1].level, LogLevel::ERROR);
    }

    #[test]
    fn test_filter_by_level() {
        let log_data = r#"{"timestamp":"2023-10-01T12:00:00Z","level":"INFO","message":"Test","component":"test"}
{"timestamp":"2023-10-01T12:01:00Z","level":"ERROR","message":"Error","component":"test"}
{"timestamp":"2023-10-01T12:02:00Z","level":"INFO","message":"Another","component":"test"}"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", log_data).unwrap();

        let parser = LogParser::new(temp_file.path().to_str().unwrap());
        let info_entries = parser.filter_by_level(LogLevel::INFO).unwrap();

        assert_eq!(info_entries.len(), 2);
        assert!(info_entries.iter().all(|e| e.level == LogLevel::INFO));
    }
}