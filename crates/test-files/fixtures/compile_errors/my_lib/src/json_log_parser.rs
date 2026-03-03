use serde_json::Value;
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
pub enum ParseError {
    IoError(std::io::Error),
    JsonError(serde_json::Error),
    InvalidFormat,
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::IoError(err)
    }
}

impl From<serde_json::Error> for ParseError {
    fn from(err: serde_json::Error) -> Self {
        ParseError::JsonError(err)
    }
}

pub struct LogParser {
    min_level: String,
    filter_key: Option<String>,
    filter_value: Option<String>,
}

impl LogParser {
    pub fn new(min_level: &str) -> Self {
        LogParser {
            min_level: min_level.to_lowercase(),
            filter_key: None,
            filter_value: None,
        }
    }

    pub fn with_filter(mut self, key: &str, value: &str) -> Self {
        self.filter_key = Some(key.to_string());
        self.filter_value = Some(value.to_string());
        self
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, ParseError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = self.parse_line(&line) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    fn parse_line(&self, line: &str) -> Result<LogEntry, ParseError> {
        let json_value: Value = serde_json::from_str(line)?;

        let timestamp = json_value["timestamp"]
            .as_str()
            .ok_or(ParseError::InvalidFormat)?
            .to_string();

        let level = json_value["level"]
            .as_str()
            .ok_or(ParseError::InvalidFormat)?
            .to_lowercase();

        if !self.is_level_allowed(&level) {
            return Err(ParseError::InvalidFormat);
        }

        if let (Some(key), Some(value)) = (&self.filter_key, &self.filter_value) {
            if let Some(actual_value) = json_value.get(key) {
                if actual_value.as_str() != Some(value) {
                    return Err(ParseError::InvalidFormat);
                }
            }
        }

        let message = json_value["message"]
            .as_str()
            .ok_or(ParseError::InvalidFormat)?
            .to_string();

        let metadata = json_value["metadata"].clone();

        Ok(LogEntry {
            timestamp,
            level,
            message,
            metadata,
        })
    }

    fn is_level_allowed(&self, level: &str) -> bool {
        let level_order = ["trace", "debug", "info", "warn", "error"];
        let min_index = level_order
            .iter()
            .position(|&l| l == self.min_level)
            .unwrap_or(0);
        let current_index = level_order.iter().position(|&l| l == level);

        current_index.map_or(false, |idx| idx >= min_index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_log() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            r#"{{"timestamp": "2024-01-15T10:30:00Z", "level": "INFO", "message": "System started", "metadata": {{"service": "api"}}}}"#
        )
        .unwrap();

        let parser = LogParser::new("info");
        let result = parser.parse_file(temp_file.path());

        assert!(result.is_ok());
        let entries = result.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].level, "info");
        assert_eq!(entries[0].message, "System started");
    }

    #[test]
    fn test_level_filtering() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            r#"{{"timestamp": "2024-01-15T10:30:00Z", "level": "DEBUG", "message": "Debug message", "metadata": {{}}}}"#
        )
        .unwrap();
        writeln!(
            temp_file,
            r#"{{"timestamp": "2024-01-15T10:31:00Z", "level": "ERROR", "message": "Error occurred", "metadata": {{}}}}"#
        )
        .unwrap();

        let parser = LogParser::new("warn");
        let result = parser.parse_file(temp_file.path());

        assert!(result.is_ok());
        let entries = result.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].level, "error");
    }

    #[test]
    fn test_metadata_filter() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            r#"{{"timestamp": "2024-01-15T10:30:00Z", "level": "INFO", "message": "Request processed", "metadata": {{"user_id": "123"}}}}"#
        )
        .unwrap();
        writeln!(
            temp_file,
            r#"{{"timestamp": "2024-01-15T10:31:00Z", "level": "INFO", "message": "Request processed", "metadata": {{"user_id": "456"}}}}"#
        )
        .unwrap();

        let parser = LogParser::new("info").with_filter("user_id", "123");
        let result = parser.parse_file(temp_file.path());

        assert!(result.is_ok());
        let entries = result.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].metadata["user_id"], "123");
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    service: String,
    message: String,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

struct LogParser {
    entries: Vec<LogEntry>,
    stats: HashMap<String, usize>,
}

impl LogParser {
    fn new() -> Self {
        LogParser {
            entries: Vec::new(),
            stats: HashMap::new(),
        }
    }

    fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = serde_json::from_str::<LogEntry>(&line) {
                self.entries.push(entry);
            }
        }
        Ok(())
    }

    fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .collect()
    }

    fn filter_by_service(&self, service: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.service == service)
            .collect()
    }

    fn generate_stats(&mut self) {
        self.stats.clear();
        for entry in &self.entries {
            *self.stats.entry(entry.level.clone()).or_insert(0) += 1;
        }
    }

    fn get_level_distribution(&self) -> &HashMap<String, usize> {
        &self.stats
    }

    fn search_messages(&self, keyword: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.message.contains(keyword))
            .collect()
    }

    fn get_earliest_timestamp(&self) -> Option<&str> {
        self.entries
            .iter()
            .min_by_key(|entry| &entry.timestamp)
            .map(|entry| entry.timestamp.as_str())
    }

    fn get_latest_timestamp(&self) -> Option<&str> {
        self.entries
            .iter()
            .max_by_key(|entry| &entry.timestamp)
            .map(|entry| entry.timestamp.as_str())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = LogParser::new();
    
    parser.load_from_file("logs.jsonl")?;
    parser.generate_stats();

    println!("Total entries: {}", parser.entries.len());
    println!("Level distribution: {:?}", parser.get_level_distribution());
    
    if let Some(earliest) = parser.get_earliest_timestamp() {
        println!("Earliest log: {}", earliest);
    }
    
    if let Some(latest) = parser.get_latest_timestamp() {
        println!("Latest log: {}", latest);
    }

    let error_logs = parser.filter_by_level("error");
    println!("Error logs count: {}", error_logs.len());

    let api_service_logs = parser.filter_by_service("api");
    println!("API service logs count: {}", api_service_logs.len());

    let search_results = parser.search_messages("timeout");
    println!("Logs containing 'timeout': {}", search_results.len());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_log_parsing() {
        let mut parser = LogParser::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        
        let log_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","service":"api","message":"Request processed","duration":150}
{"timestamp":"2024-01-15T10:31:00Z","level":"ERROR","service":"db","message":"Connection timeout","attempt":3}
{"timestamp":"2024-01-15T10:32:00Z","level":"WARN","service":"api","message":"High latency detected"}"#;
        
        writeln!(temp_file, "{}", log_data).unwrap();
        
        parser.load_from_file(temp_file.path()).unwrap();
        assert_eq!(parser.entries.len(), 3);
    }

    #[test]
    fn test_filter_by_level() {
        let mut parser = LogParser::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        
        let log_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","service":"api","message":"Test"}
{"timestamp":"2024-01-15T10:31:00Z","level":"ERROR","service":"db","message":"Test"}"#;
        
        writeln!(temp_file, "{}", log_data).unwrap();
        parser.load_from_file(temp_file.path()).unwrap();
        
        let errors = parser.filter_by_level("ERROR");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].level, "ERROR");
    }

    #[test]
    fn test_stats_generation() {
        let mut parser = LogParser::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        
        let log_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","service":"api","message":"Test"}
{"timestamp":"2024-01-15T10:31:00Z","level":"ERROR","service":"db","message":"Test"}
{"timestamp":"2024-01-15T10:32:00Z","level":"INFO","service":"cache","message":"Test"}"#;
        
        writeln!(temp_file, "{}", log_data).unwrap();
        parser.load_from_file(temp_file.path()).unwrap();
        parser.generate_stats();
        
        let stats = parser.get_level_distribution();
        assert_eq!(stats.get("INFO"), Some(&2));
        assert_eq!(stats.get("ERROR"), Some(&1));
    }
}