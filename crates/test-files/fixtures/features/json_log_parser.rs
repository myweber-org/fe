use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    service: String,
    message: String,
    metadata: HashMap<String, String>,
}

struct LogParser {
    entries: Vec<LogEntry>,
}

impl LogParser {
    fn new() -> Self {
        LogParser { entries: Vec::new() }
    }

    fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<LogEntry>(&line) {
                Ok(entry) => self.entries.push(entry),
                Err(e) => eprintln!("Failed to parse line: {} - {}", line, e),
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

    fn count_by_level(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for entry in &self.entries {
            *counts.entry(entry.level.clone()).or_insert(0) += 1;
        }
        counts
    }

    fn count_by_service(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for entry in &self.entries {
            *counts.entry(entry.service.clone()).or_insert(0) += 1;
        }
        counts
    }

    fn search_messages(&self, keyword: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.message.to_lowercase().contains(&keyword.to_lowercase()))
            .collect()
    }

    fn get_time_range(&self) -> Option<(String, String)> {
        if self.entries.is_empty() {
            return None;
        }

        let mut timestamps: Vec<&String> = self.entries.iter().map(|e| &e.timestamp).collect();
        timestamps.sort();

        Some((timestamps[0].clone(), timestamps[timestamps.len() - 1].clone()))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = LogParser::new();
    
    parser.load_from_file("logs.jsonl")?;
    
    println!("Total entries: {}", parser.entries.len());
    
    let error_logs = parser.filter_by_level("ERROR");
    println!("Error logs: {}", error_logs.len());
    
    let level_counts = parser.count_by_level();
    println!("Log level distribution:");
    for (level, count) in &level_counts {
        println!("  {}: {}", level, count);
    }
    
    if let Some((start, end)) = parser.get_time_range() {
        println!("Time range: {} - {}", start, end);
    }
    
    let search_results = parser.search_messages("timeout");
    println!("Found {} entries containing 'timeout'", search_results.len());
    
    Ok(())
}use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LogParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parse error at line {line}: {source}")]
    JsonParse {
        line: usize,
        source: serde_json::Error,
    },
    #[error("Missing required field '{field}' at line {line}")]
    MissingField { line: usize, field: String },
}

pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub metadata: Value,
}

pub fn parse_log_file(path: &str) -> Result<Vec<LogEntry>, LogParseError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result?;
        let line_number = line_num + 1;

        let json_value: Value = serde_json::from_str(&line)
            .map_err(|e| LogParseError::JsonParse {
                line: line_number,
                source: e,
            })?;

        let entry = parse_log_entry(json_value, line_number)?;
        entries.push(entry);
    }

    Ok(entries)
}

fn parse_log_entry(value: Value, line_number: usize) -> Result<LogEntry, LogParseError> {
    let obj = value.as_object().ok_or_else(|| LogParseError::MissingField {
        line: line_number,
        field: "object".to_string(),
    })?;

    let timestamp = obj
        .get("timestamp")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LogParseError::MissingField {
            line: line_number,
            field: "timestamp".to_string(),
        })?
        .to_string();

    let level = obj
        .get("level")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LogParseError::MissingField {
            line: line_number,
            field: "level".to_string(),
        })?
        .to_string();

    let message = obj
        .get("message")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LogParseError::MissingField {
            line: line_number,
            field: "message".to_string(),
        })?
        .to_string();

    let metadata = obj
        .get("metadata")
        .cloned()
        .unwrap_or_else(|| Value::Object(serde_json::Map::new()));

    Ok(LogEntry {
        timestamp,
        level,
        message,
        metadata,
    })
}

pub fn filter_by_level(entries: &[LogEntry], level: &str) -> Vec<&LogEntry> {
    entries
        .iter()
        .filter(|entry| entry.level.eq_ignore_ascii_case(level))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_log() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let log_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","message":"Service started","metadata":{"pid":1234}}
{"timestamp":"2024-01-15T10:31:00Z","level":"ERROR","message":"Connection failed","metadata":{"retry_count":3}}"#;
        write!(temp_file, "{}", log_data).unwrap();

        let entries = parse_log_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].level, "INFO");
        assert_eq!(entries[1].level, "ERROR");
    }

    #[test]
    fn test_filter_by_level() {
        let entries = vec![
            LogEntry {
                timestamp: "2024-01-15T10:30:00Z".to_string(),
                level: "INFO".to_string(),
                message: "Test info".to_string(),
                metadata: json!({}),
            },
            LogEntry {
                timestamp: "2024-01-15T10:31:00Z".to_string(),
                level: "ERROR".to_string(),
                message: "Test error".to_string(),
                metadata: json!({}),
            },
            LogEntry {
                timestamp: "2024-01-15T10:32:00Z".to_string(),
                level: "INFO".to_string(),
                message: "Another info".to_string(),
                metadata: json!({}),
            },
        ];

        let errors = filter_by_level(&entries, "ERROR");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "Test error");
    }
}