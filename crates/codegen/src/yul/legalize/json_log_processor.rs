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
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum LogLevel {
    DEBUG,
    INFO,
    WARN,
    ERROR,
    CRITICAL,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub service: String,
    pub message: String,
    pub metadata: HashMap<String, String>,
}

pub struct LogProcessor {
    min_level: LogLevel,
    service_filter: Option<String>,
}

impl LogProcessor {
    pub fn new(min_level: LogLevel) -> Self {
        LogProcessor {
            min_level,
            service_filter: None,
        }
    }

    pub fn with_service_filter(mut self, service_name: &str) -> Self {
        self.service_filter = Some(service_name.to_string());
        self
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line_content = line.map_err(|e| format!("Line {} read error: {}", line_num + 1, e))?;
            
            match serde_json::from_str::<LogEntry>(&line_content) {
                Ok(mut entry) => {
                    if self.should_include(&entry) {
                        self.enrich_entry(&mut entry);
                        entries.push(entry);
                    }
                }
                Err(e) => eprintln!("Warning: Failed to parse line {}: {}", line_num + 1, e),
            }
        }

        Ok(entries)
    }

    fn should_include(&self, entry: &LogEntry) -> bool {
        if self.level_to_numeric(&entry.level) < self.level_to_numeric(&self.min_level) {
            return false;
        }

        if let Some(ref service) = self.service_filter {
            if entry.service != *service {
                return false;
            }
        }

        true
    }

    fn level_to_numeric(&self, level: &LogLevel) -> u8 {
        match level {
            LogLevel::DEBUG => 1,
            LogLevel::INFO => 2,
            LogLevel::WARN => 3,
            LogLevel::ERROR => 4,
            LogLevel::CRITICAL => 5,
        }
    }

    fn enrich_entry(&self, entry: &mut LogEntry) {
        entry.metadata.insert("processed_timestamp".to_string(), 
            chrono::Utc::now().to_rfc3339());
    }

    pub fn count_by_level(&self, entries: &[LogEntry]) -> HashMap<LogLevel, usize> {
        let mut counts = HashMap::new();
        
        for entry in entries {
            *counts.entry(entry.level.clone()).or_insert(0) += 1;
        }
        
        counts
    }
}

pub fn export_to_jsonl(entries: &[LogEntry], output_path: &str) -> Result<(), String> {
    let file = File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    let mut writer = std::io::BufWriter::new(file);

    for entry in entries {
        let json = serde_json::to_string(entry)
            .map_err(|e| format!("Failed to serialize entry: {}", e))?;
        writeln!(writer, "{}", json)
            .map_err(|e| format!("Failed to write to file: {}", e))?;
    }

    Ok(())
}