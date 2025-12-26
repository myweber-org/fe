use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogLevel {
    ERROR,
    WARN,
    INFO,
    DEBUG,
    TRACE,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub message: String,
    pub fields: HashMap<String, String>,
}

pub struct LogProcessor {
    min_level: LogLevel,
    include_fields: Vec<String>,
}

impl LogProcessor {
    pub fn new(min_level: LogLevel, include_fields: Vec<String>) -> Self {
        Self {
            min_level,
            include_fields,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| format!("Line {} read error: {}", line_num + 1, e))?;
            
            match self.parse_line(&line) {
                Ok(Some(entry)) => entries.push(entry),
                Ok(None) => continue,
                Err(e) => eprintln!("Line {} parse error: {}", line_num + 1, e),
            }
        }

        Ok(entries)
    }

    fn parse_line(&self, line: &str) -> Result<Option<LogEntry>, String> {
        let entry: LogEntry = serde_json::from_str(line)
            .map_err(|e| format!("JSON parse error: {}", e))?;

        if !self.should_include(&entry.level) {
            return Ok(None);
        }

        let filtered_entry = self.filter_fields(entry);
        Ok(Some(filtered_entry))
    }

    fn should_include(&self, level: &LogLevel) -> bool {
        match (&self.min_level, level) {
            (LogLevel::ERROR, _) => matches!(level, LogLevel::ERROR),
            (LogLevel::WARN, _) => matches!(level, LogLevel::ERROR | LogLevel::WARN),
            (LogLevel::INFO, _) => matches!(level, LogLevel::ERROR | LogLevel::WARN | LogLevel::INFO),
            (LogLevel::DEBUG, _) => matches!(level, LogLevel::ERROR | LogLevel::WARN | LogLevel::INFO | LogLevel::DEBUG),
            (LogLevel::TRACE, _) => true,
        }
    }

    fn filter_fields(&self, mut entry: LogEntry) -> LogEntry {
        if self.include_fields.is_empty() {
            return entry;
        }

        let mut filtered_fields = HashMap::new();
        for field in &self.include_fields {
            if let Some(value) = entry.fields.get(field) {
                filtered_fields.insert(field.clone(), value.clone());
            }
        }
        entry.fields = filtered_fields;
        entry
    }

    pub fn count_by_level(&self, entries: &[LogEntry]) -> HashMap<LogLevel, usize> {
        let mut counts = HashMap::new();
        for entry in entries {
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
    fn test_log_processing() {
        let log_data = r#"{"timestamp": "2024-01-15T10:30:00Z", "level": "ERROR", "message": "Database connection failed", "fields": {"service": "api", "error_code": "DB_001"}}
{"timestamp": "2024-01-15T10:31:00Z", "level": "INFO", "message": "User login successful", "fields": {"user_id": "12345", "service": "auth"}}"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", log_data).unwrap();

        let processor = LogProcessor::new(LogLevel::INFO, vec!["service".to_string()]);
        let entries = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].level, LogLevel::ERROR);
        assert_eq!(entries[1].level, LogLevel::INFO);
        assert!(entries[0].fields.contains_key("service"));
        assert!(!entries[0].fields.contains_key("error_code"));
    }

    #[test]
    fn test_level_filtering() {
        let processor = LogProcessor::new(LogLevel::WARN, vec![]);
        
        let test_entries = vec![
            LogEntry {
                timestamp: "2024-01-15T10:30:00Z".to_string(),
                level: LogLevel::ERROR,
                message: "Test error".to_string(),
                fields: HashMap::new(),
            },
            LogEntry {
                timestamp: "2024-01-15T10:31:00Z".to_string(),
                level: LogLevel::INFO,
                message: "Test info".to_string(),
                fields: HashMap::new(),
            },
        ];

        let filtered: Vec<LogEntry> = test_entries
            .into_iter()
            .filter(|e| processor.should_include(&e.level))
            .collect();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].level, LogLevel::ERROR);
    }
}