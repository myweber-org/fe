
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
    pub message: String,
    pub metadata: HashMap<String, String>,
}

pub struct LogProcessor {
    min_level: LogLevel,
    include_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
}

impl LogProcessor {
    pub fn new(min_level: LogLevel) -> Self {
        LogProcessor {
            min_level,
            include_patterns: Vec::new(),
            exclude_patterns: Vec::new(),
        }
    }

    pub fn add_include_pattern(&mut self, pattern: String) {
        self.include_patterns.push(pattern);
    }

    pub fn add_exclude_pattern(&mut self, pattern: String) {
        self.exclude_patterns.push(pattern);
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);
        let mut filtered_logs = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line_content = line.map_err(|e| format!("Failed to read line {}: {}", line_num + 1, e))?;
            
            if let Ok(log_entry) = serde_json::from_str::<LogEntry>(&line_content) {
                if self.should_include(&log_entry) {
                    filtered_logs.push(log_entry);
                }
            }
        }

        Ok(filtered_logs)
    }

    fn should_include(&self, entry: &LogEntry) -> bool {
        if !self.meets_level_requirement(entry) {
            return false;
        }

        if !self.include_patterns.is_empty() && !self.matches_any_pattern(&entry.message, &self.include_patterns) {
            return false;
        }

        if self.matches_any_pattern(&entry.message, &self.exclude_patterns) {
            return false;
        }

        true
    }

    fn meets_level_requirement(&self, entry: &LogEntry) -> bool {
        match (&self.min_level, &entry.level) {
            (LogLevel::DEBUG, _) => true,
            (LogLevel::INFO, LogLevel::DEBUG) => false,
            (LogLevel::INFO, _) => true,
            (LogLevel::WARN, LogLevel::DEBUG | LogLevel::INFO) => false,
            (LogLevel::WARN, _) => true,
            (LogLevel::ERROR, LogLevel::DEBUG | LogLevel::INFO | LogLevel::WARN) => false,
            (LogLevel::ERROR, _) => true,
            (LogLevel::CRITICAL, LogLevel::CRITICAL) => true,
            (LogLevel::CRITICAL, _) => false,
        }
    }

    fn matches_any_pattern(&self, text: &str, patterns: &[String]) -> bool {
        patterns.iter().any(|pattern| text.contains(pattern))
    }
}

pub fn count_logs_by_level(logs: &[LogEntry]) -> HashMap<LogLevel, usize> {
    let mut counts = HashMap::new();
    
    for log in logs {
        *counts.entry(log.level.clone()).or_insert(0) += 1;
    }
    
    counts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_filtering() {
        let mut processor = LogProcessor::new(LogLevel::WARN);
        
        let test_log = LogEntry {
            timestamp: "2024-01-15T10:30:00Z".to_string(),
            level: LogLevel::INFO,
            message: "Test message".to_string(),
            metadata: HashMap::new(),
        };

        assert!(!processor.should_include(&test_log));
    }

    #[test]
    fn test_pattern_filtering() {
        let mut processor = LogProcessor::new(LogLevel::DEBUG);
        processor.add_include_pattern("error".to_string());
        
        let test_log = LogEntry {
            timestamp: "2024-01-15T10:30:00Z".to_string(),
            level: LogLevel::ERROR,
            message: "Database connection error".to_string(),
            metadata: HashMap::new(),
        };

        assert!(processor.should_include(&test_log));
    }
}