use std::fs::File;
use std::io::{BufRead, BufReader};
use chrono::{DateTime, FixedOffset};
use serde_json::Value;

#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: DateTime<FixedOffset>,
    pub level: String,
    pub message: String,
    pub raw_data: Value,
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

    pub fn parse_with_filters(
        &self,
        min_level: Option<&str>,
        start_time: Option<DateTime<FixedOffset>>,
        end_time: Option<DateTime<FixedOffset>>,
    ) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let json_value: Value = serde_json::from_str(&line)?;

            let timestamp_str = json_value["timestamp"]
                .as_str()
                .ok_or("Missing timestamp field")?;
            let timestamp = DateTime::parse_from_rfc3339(timestamp_str)?;

            if let Some(start) = start_time {
                if timestamp < start {
                    continue;
                }
            }

            if let Some(end) = end_time {
                if timestamp > end {
                    continue;
                }
            }

            let level = json_value["level"]
                .as_str()
                .ok_or("Missing level field")?
                .to_string();

            if let Some(min_lvl) = min_level {
                if !self.is_level_allowed(&level, min_lvl) {
                    continue;
                }
            }

            let message = json_value["message"]
                .as_str()
                .unwrap_or("")
                .to_string();

            entries.push(LogEntry {
                timestamp,
                level,
                message,
                raw_data: json_value,
            });
        }

        Ok(entries)
    }

    fn is_level_allowed(&self, entry_level: &str, min_level: &str) -> bool {
        let levels = ["trace", "debug", "info", "warn", "error"];
        let entry_idx = levels.iter().position(|&l| l == entry_level.to_lowercase());
        let min_idx = levels.iter().position(|&l| l == min_level.to_lowercase());

        match (entry_idx, min_idx) {
            (Some(e), Some(m)) => e >= m,
            _ => false,
        }
    }

    pub fn count_by_level(&self) -> Result<std::collections::HashMap<String, usize>, Box<dyn std::error::Error>> {
        let entries = self.parse_with_filters(None, None, None)?;
        let mut counts = std::collections::HashMap::new();

        for entry in entries {
            *counts.entry(entry.level).or_insert(0) += 1;
        }

        Ok(counts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_level_filtering() {
        let parser = LogParser::new("test_logs.json");
        let start_time = FixedOffset::east_opt(3600)
            .unwrap()
            .with_ymd_and_hms(2024, 1, 1, 0, 0, 0)
            .unwrap();
        
        let result = parser.parse_with_filters(Some("info"), Some(start_time), None);
        assert!(result.is_ok());
    }
}
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    fn from_string(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "error" => Some(LogLevel::Error),
            "warn" => Some(LogLevel::Warn),
            "info" => Some(LogLevel::Info),
            "debug" => Some(LogLevel::Debug),
            "trace" => Some(LogLevel::Trace),
            _ => None,
        }
    }
}

pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub message: String,
    pub fields: HashMap<String, Value>,
}

pub struct LogParser {
    min_level: LogLevel,
}

impl LogParser {
    pub fn new(min_level: LogLevel) -> Self {
        LogParser { min_level }
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);
        
        let mut entries = Vec::new();
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| format!("Failed to read line {}: {}", line_num + 1, e))?;
            
            if let Some(entry) = self.parse_line(&line) {
                entries.push(entry);
            }
        }
        
        Ok(entries)
    }

    fn parse_line(&self, line: &str) -> Option<LogEntry> {
        let json_value: Value = serde_json::from_str(line).ok()?;
        
        let obj = json_value.as_object()?;
        
        let timestamp = obj.get("timestamp")?.as_str()?.to_string();
        let level_str = obj.get("level")?.as_str()?;
        let level = LogLevel::from_string(level_str)?;
        
        if !self.should_include(&level) {
            return None;
        }
        
        let message = obj.get("message")?.as_str()?.to_string();
        
        let mut fields = HashMap::new();
        for (key, value) in obj {
            if key != "timestamp" && key != "level" && key != "message" {
                fields.insert(key.clone(), value.clone());
            }
        }
        
        Some(LogEntry {
            timestamp,
            level,
            message,
            fields,
        })
    }

    fn should_include(&self, level: &LogLevel) -> bool {
        match (&self.min_level, level) {
            (LogLevel::Error, _) => matches!(level, LogLevel::Error),
            (LogLevel::Warn, _) => matches!(level, LogLevel::Error | LogLevel::Warn),
            (LogLevel::Info, _) => matches!(level, LogLevel::Error | LogLevel::Warn | LogLevel::Info),
            (LogLevel::Debug, _) => matches!(level, LogLevel::Error | LogLevel::Warn | LogLevel::Info | LogLevel::Debug),
            (LogLevel::Trace, _) => true,
        }
    }
}

pub fn count_entries_by_level(entries: &[LogEntry]) -> HashMap<LogLevel, usize> {
    let mut counts = HashMap::new();
    
    for entry in entries {
        *counts.entry(entry.level).or_insert(0) += 1;
    }
    
    counts
}