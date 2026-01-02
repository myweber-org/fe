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
    pub fn from_str(level: &str) -> Option<Self> {
        match level.to_uppercase().as_str() {
            "DEBUG" => Some(LogLevel::DEBUG),
            "INFO" => Some(LogLevel::INFO),
            "WARN" => Some(LogLevel::WARN),
            "ERROR" => Some(LogLevel::ERROR),
            "CRITICAL" => Some(LogLevel::CRITICAL),
            _ => None,
        }
    }

    pub fn severity(&self) -> u8 {
        match self {
            LogLevel::DEBUG => 1,
            LogLevel::INFO => 2,
            LogLevel::WARN => 3,
            LogLevel::ERROR => 4,
            LogLevel::CRITICAL => 5,
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
    include_modules: Vec<String>,
    exclude_modules: Vec<String>,
}

impl LogProcessor {
    pub fn new(min_level: LogLevel) -> Self {
        Self {
            min_level,
            include_modules: Vec::new(),
            exclude_modules: Vec::new(),
        }
    }

    pub fn include_module(mut self, module: &str) -> Self {
        self.include_modules.push(module.to_string());
        self
    }

    pub fn exclude_module(mut self, module: &str) -> Self {
        self.exclude_modules.push(module.to_string());
        self
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);
        let mut filtered_logs = Vec::new();

        for line in reader.lines() {
            let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
            
            if let Ok(log_entry) = serde_json::from_str::<LogEntry>(&line) {
                if self.should_include(&log_entry) {
                    filtered_logs.push(log_entry);
                }
            }
        }

        Ok(filtered_logs)
    }

    fn should_include(&self, entry: &LogEntry) -> bool {
        if entry.level.severity() < self.min_level.severity() {
            return false;
        }

        if let Some(module) = &entry.module {
            if !self.include_modules.is_empty() && !self.include_modules.contains(module) {
                return false;
            }

            if self.exclude_modules.contains(module) {
                return false;
            }
        }

        true
    }

    pub fn count_by_level(&self, logs: &[LogEntry]) -> HashMap<LogLevel, usize> {
        let mut counts = HashMap::new();
        
        for log in logs {
            *counts.entry(log.level.clone()).or_insert(0) += 1;
        }
        
        counts
    }
}

pub fn find_errors(logs: &[LogEntry]) -> Vec<&LogEntry> {
    logs.iter()
        .filter(|log| {
            log.level == LogLevel::ERROR || log.level == LogLevel::CRITICAL
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_log() -> LogEntry {
        let mut metadata = HashMap::new();
        metadata.insert("user_id".to_string(), "12345".to_string());
        metadata.insert("request_id".to_string(), "abc-123".to_string());

        LogEntry {
            timestamp: "2024-01-15T10:30:00Z".to_string(),
            level: LogLevel::ERROR,
            message: "Database connection failed".to_string(),
            module: Some("database".to_string()),
            metadata,
        }
    }

    #[test]
    fn test_log_level_severity() {
        assert_eq!(LogLevel::DEBUG.severity(), 1);
        assert_eq!(LogLevel::INFO.severity(), 2);
        assert_eq!(LogLevel::ERROR.severity(), 4);
    }

    #[test]
    fn test_log_level_from_str() {
        assert_eq!(LogLevel::from_str("error"), Some(LogLevel::ERROR));
        assert_eq!(LogLevel::from_str("INFO"), Some(LogLevel::INFO));
        assert_eq!(LogLevel::from_str("invalid"), None);
    }

    #[test]
    fn test_should_include() {
        let processor = LogProcessor::new(LogLevel::INFO)
            .include_module("database")
            .exclude_module("auth");

        let mut log = create_test_log();
        
        log.level = LogLevel::DEBUG;
        assert!(!processor.should_include(&log));
        
        log.level = LogLevel::INFO;
        log.module = Some("auth".to_string());
        assert!(!processor.should_include(&log));
        
        log.module = Some("database".to_string());
        assert!(processor.should_include(&log));
    }

    #[test]
    fn test_find_errors() {
        let logs = vec![
            create_test_log(),
            LogEntry {
                timestamp: "2024-01-15T10:31:00Z".to_string(),
                level: LogLevel::INFO,
                message: "User logged in".to_string(),
                module: Some("auth".to_string()),
                metadata: HashMap::new(),
            },
        ];

        let errors = find_errors(&logs);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].level, LogLevel::ERROR);
    }
}