use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub enum LogLevel {
    DEBUG,
    INFO,
    WARN,
    ERROR,
    CRITICAL,
}

impl LogLevel {
    fn severity(&self) -> u8 {
        match self {
            LogLevel::DEBUG => 1,
            LogLevel::INFO => 2,
            LogLevel::WARN => 3,
            LogLevel::ERROR => 4,
            LogLevel::CRITICAL => 5,
        }
    }

    fn from_str(level: &str) -> Option<Self> {
        match level.to_uppercase().as_str() {
            "DEBUG" => Some(LogLevel::DEBUG),
            "INFO" => Some(LogLevel::INFO),
            "WARN" => Some(LogLevel::WARN),
            "ERROR" => Some(LogLevel::ERROR),
            "CRITICAL" => Some(LogLevel::CRITICAL),
            _ => None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub message: String,
    pub module: Option<String>,
    pub metadata: HashMap<String, String>,
}

pub struct LogProcessor {
    min_level: LogLevel,
    filters: Vec<String>,
}

impl LogProcessor {
    pub fn new(min_level: LogLevel) -> Self {
        LogProcessor {
            min_level,
            filters: Vec::new(),
        }
    }

    pub fn add_filter(&mut self, filter: String) {
        self.filters.push(filter);
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
            
            if let Ok(entry) = serde_json::from_str::<LogEntry>(&line) {
                if self.should_process(&entry) {
                    entries.push(entry);
                }
            }
        }

        Ok(entries)
    }

    fn should_process(&self, entry: &LogEntry) -> bool {
        if entry.level.severity() < self.min_level.severity() {
            return false;
        }

        if !self.filters.is_empty() {
            for filter in &self.filters {
                if entry.message.contains(filter) {
                    return true;
                }
                if let Some(module) = &entry.module {
                    if module.contains(filter) {
                        return true;
                    }
                }
            }
            return false;
        }

        true
    }

    pub fn group_by_level(&self, entries: &[LogEntry]) -> HashMap<LogLevel, Vec<&LogEntry>> {
        let mut grouped = HashMap::new();
        
        for entry in entries {
            grouped.entry(entry.level.clone())
                .or_insert_with(Vec::new)
                .push(entry);
        }
        
        grouped
    }

    pub fn extract_metadata_keys(&self, entries: &[LogEntry]) -> Vec<String> {
        let mut keys = std::collections::HashSet::new();
        
        for entry in entries {
            for key in entry.metadata.keys() {
                keys.insert(key.clone());
            }
        }
        
        keys.into_iter().collect()
    }
}

pub fn parse_log_level(level_str: &str) -> Result<LogLevel, String> {
    LogLevel::from_str(level_str)
        .ok_or_else(|| format!("Invalid log level: {}", level_str))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_processor_filtering() {
        let mut processor = LogProcessor::new(LogLevel::INFO);
        processor.add_filter("database".to_string());

        let mut temp_file = NamedTempFile::new().unwrap();
        let log_data = r#"{"timestamp": "2024-01-15T10:30:00Z", "level": "INFO", "message": "Database connection established", "module": "db", "metadata": {"connection_id": "123"}}
{"timestamp": "2024-01-15T10:31:00Z", "level": "DEBUG", "message": "Cache miss", "module": "cache", "metadata": {"key": "user:42"}}
{"timestamp": "2024-01-15T10:32:00Z", "level": "ERROR", "message": "Query failed", "module": "db", "metadata": {"query": "SELECT * FROM users"}}"#;
        
        write!(temp_file, "{}", log_data).unwrap();
        
        let entries = processor.process_file(temp_file.path()).unwrap();
        assert_eq!(entries.len(), 2);
        
        let grouped = processor.group_by_level(&entries);
        assert_eq!(grouped.get(&LogLevel::INFO).unwrap().len(), 1);
        assert_eq!(grouped.get(&LogLevel::ERROR).unwrap().len(), 1);
    }

    #[test]
    fn test_parse_log_level() {
        assert_eq!(parse_log_level("INFO").unwrap(), LogLevel::INFO);
        assert_eq!(parse_log_level("error").unwrap(), LogLevel::ERROR);
        assert!(parse_log_level("INVALID").is_err());
    }
}