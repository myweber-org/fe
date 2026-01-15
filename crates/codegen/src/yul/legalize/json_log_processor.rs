use serde::Deserialize;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Deserialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    module: Option<String>,
}

#[derive(Debug)]
pub struct LogProcessor {
    min_level: LogLevel,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

impl LogProcessor {
    pub fn new(min_level: LogLevel) -> Self {
        LogProcessor { min_level }
    }

    pub fn process_file(&self, path: &str) -> Result<Vec<LogEntry>, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);
        let mut filtered_entries = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line_content = line.map_err(|e| format!("Line {} read error: {}", line_num + 1, e))?;
            
            match serde_json::from_str::<LogEntry>(&line_content) {
                Ok(entry) => {
                    if self.should_include(&entry.level) {
                        filtered_entries.push(entry);
                    }
                }
                Err(e) => eprintln!("Line {} parse error: {}", line_num + 1, e),
            }
        }

        Ok(filtered_entries)
    }

    fn should_include(&self, level_str: &str) -> bool {
        let entry_level = match level_str.to_lowercase().as_str() {
            "trace" => LogLevel::Trace,
            "debug" => LogLevel::Debug,
            "info" => LogLevel::Info,
            "warn" => LogLevel::Warn,
            "error" => LogLevel::Error,
            "fatal" => LogLevel::Fatal,
            _ => return false,
        };

        entry_level >= self.min_level
    }
}

impl LogLevel {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "trace" => Some(LogLevel::Trace),
            "debug" => Some(LogLevel::Debug),
            "info" => Some(LogLevel::Info),
            "warn" => Some(LogLevel::Warn),
            "error" => Some(LogLevel::Error),
            "fatal" => Some(LogLevel::Fatal),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_filtering() {
        let logs = r#"{"timestamp": "2024-01-01T00:00:00Z", "level": "INFO", "message": "System started"}
{"timestamp": "2024-01-01T00:00:01Z", "level": "DEBUG", "message": "Debug data"}
{"timestamp": "2024-01-01T00:00:02Z", "level": "ERROR", "message": "Critical failure"}"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", logs).unwrap();
        
        let processor = LogProcessor::new(LogLevel::Info);
        let result = processor.process_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|e| e.level == "INFO"));
        assert!(result.iter().any(|e| e.level == "ERROR"));
        assert!(!result.iter().any(|e| e.level == "DEBUG"));
    }

    #[test]
    fn test_level_ordering() {
        assert!(LogLevel::Error > LogLevel::Warn);
        assert!(LogLevel::Info >= LogLevel::Info);
        assert!(LogLevel::Trace < LogLevel::Fatal);
    }
}