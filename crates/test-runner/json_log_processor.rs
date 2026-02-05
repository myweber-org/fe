
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug)]
struct LogSummary {
    total_entries: usize,
    error_count: usize,
    warning_count: usize,
    info_count: usize,
    unique_messages: HashMap<String, usize>,
}

impl LogSummary {
    fn new() -> Self {
        LogSummary {
            total_entries: 0,
            error_count: 0,
            warning_count: 0,
            info_count: 0,
            unique_messages: HashMap::new(),
        }
    }

    fn update(&mut self, entry: &LogEntry) {
        self.total_entries += 1;
        
        match entry.level.as_str() {
            "ERROR" => self.error_count += 1,
            "WARNING" => self.warning_count += 1,
            "INFO" => self.info_count += 1,
            _ => (),
        }
        
        *self.unique_messages
            .entry(entry.message.clone())
            .or_insert(0) += 1;
    }
}

fn parse_log_file<P: AsRef<Path>>(path: P) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<LogEntry>(&line) {
            Ok(entry) => entries.push(entry),
            Err(e) => eprintln!("Failed to parse line: {}\nError: {}", line, e),
        }
    }

    Ok(entries)
}

fn filter_logs_by_level(entries: &[LogEntry], level: &str) -> Vec<LogEntry> {
    entries
        .iter()
        .filter(|entry| entry.level == level)
        .cloned()
        .collect()
}

fn generate_summary(entries: &[LogEntry]) -> LogSummary {
    let mut summary = LogSummary::new();
    
    for entry in entries {
        summary.update(entry);
    }
    
    summary
}

fn save_filtered_logs<P: AsRef<Path>>(
    entries: &[LogEntry],
    path: P,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(path)?;
    let mut writer = std::io::BufWriter::new(file);
    
    for entry in entries {
        let json = serde_json::to_string(entry)?;
        writeln!(writer, "{}", json)?;
    }
    
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log_file = "application.log";
    
    println!("Processing log file: {}", log_file);
    
    let entries = parse_log_file(log_file)?;
    println!("Total log entries parsed: {}", entries.len());
    
    let summary = generate_summary(&entries);
    println!("Log Summary:");
    println!("  Total entries: {}", summary.total_entries);
    println!("  Errors: {}", summary.error_count);
    println!("  Warnings: {}", summary.warning_count);
    println!("  Info messages: {}", summary.info_count);
    println!("  Unique message types: {}", summary.unique_messages.len());
    
    let error_logs = filter_logs_by_level(&entries, "ERROR");
    if !error_logs.is_empty() {
        println!("\nFound {} error logs:", error_logs.len());
        save_filtered_logs(&error_logs, "errors.json")?;
        println!("Error logs saved to errors.json");
    }
    
    let top_messages: Vec<(&String, &usize)> = summary
        .unique_messages
        .iter()
        .take(5)
        .collect();
    
    if !top_messages.is_empty() {
        println!("\nTop 5 most frequent messages:");
        for (message, count) in top_messages {
            println!("  {} ({} occurrences)", message, count);
        }
    }
    
    Ok(())
}
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LogError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Missing required field: {0}")]
    MissingField(String),
}

pub struct LogProcessor {
    field_filters: HashMap<String, String>,
    required_fields: Vec<String>,
}

impl LogProcessor {
    pub fn new() -> Self {
        LogProcessor {
            field_filters: HashMap::new(),
            required_fields: Vec::new(),
        }
    }

    pub fn add_filter(&mut self, field: &str, value: &str) {
        self.field_filters.insert(field.to_string(), value.to_string());
    }

    pub fn require_field(&mut self, field: &str) {
        self.required_fields.push(field.to_string());
    }

    pub fn process_file(&self, path: &str) -> Result<Vec<Value>, LogError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut results = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            match self.process_line(&line) {
                Ok(Some(value)) => results.push(value),
                Ok(None) => continue,
                Err(e) => eprintln!("Line {} error: {}", line_num + 1, e),
            }
        }

        Ok(results)
    }

    fn process_line(&self, line: &str) -> Result<Option<Value>, LogError> {
        let parsed: Value = serde_json::from_str(line)?;
        
        for field in &self.required_fields {
            if !parsed.get(field).is_some() {
                return Err(LogError::MissingField(field.clone()));
            }
        }

        for (field, expected) in &self.field_filters {
            if let Some(value) = parsed.get(field) {
                if value.as_str() != Some(expected) {
                    return Ok(None);
                }
            } else {
                return Ok(None);
            }
        }

        Ok(Some(parsed))
    }

    pub fn extract_fields(&self, logs: &[Value], fields: &[&str]) -> Vec<HashMap<String, String>> {
        logs.iter()
            .filter_map(|log| {
                let mut extracted = HashMap::new();
                
                for field in fields {
                    if let Some(value) = log.get(*field) {
                        let str_value = match value {
                            Value::String(s) => s.clone(),
                            Value::Number(n) => n.to_string(),
                            Value::Bool(b) => b.to_string(),
                            _ => continue,
                        };
                        extracted.insert(field.to_string(), str_value);
                    }
                }
                
                if extracted.len() == fields.len() {
                    Some(extracted)
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_processing() {
        let mut processor = LogProcessor::new();
        processor.add_filter("level", "ERROR");
        processor.require_field("timestamp");

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{"level":"ERROR","timestamp":"2024-01-01","message":"Test error"}}"#).unwrap();
        writeln!(temp_file, r#"{{"level":"INFO","timestamp":"2024-01-01","message":"Test info"}}"#).unwrap();

        let results = processor.process_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(results.len(), 1);
        
        let extracted = processor.extract_fields(&results, &["level", "message"]);
        assert_eq!(extracted[0]["level"], "ERROR");
        assert_eq!(extracted[0]["message"], "Test error");
    }
}
use serde_json::{Value, Error as JsonError};
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

pub fn parse_log_file<P: AsRef<Path>>(path: P) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line_content = line?;
        
        match parse_log_line(&line_content) {
            Ok(entry) => entries.push(entry),
            Err(e) => eprintln!("Warning: Failed to parse line {}: {}", line_num + 1, e),
        }
    }

    Ok(entries)
}

fn parse_log_line(line: &str) -> Result<LogEntry, JsonError> {
    let json_value: Value = serde_json::from_str(line)?;
    
    let timestamp = json_value["timestamp"]
        .as_str()
        .unwrap_or("unknown")
        .to_string();
    
    let level = json_value["level"]
        .as_str()
        .unwrap_or("INFO")
        .to_string();
    
    let message = json_value["message"]
        .as_str()
        .unwrap_or("")
        .to_string();
    
    let metadata = json_value.get("metadata")
        .cloned()
        .unwrap_or(Value::Null);

    Ok(LogEntry {
        timestamp,
        level,
        message,
        metadata,
    })
}

pub fn filter_logs_by_level(entries: &[LogEntry], level: &str) -> Vec<&LogEntry> {
    entries.iter()
        .filter(|entry| entry.level.to_uppercase() == level.to_uppercase())
        .collect()
}

pub fn count_logs_by_level(entries: &[LogEntry]) -> std::collections::HashMap<String, usize> {
    let mut counts = std::collections::HashMap::new();
    
    for entry in entries {
        *counts.entry(entry.level.clone()).or_insert(0) += 1;
    }
    
    counts
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_valid_log_line() {
        let json_line = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"ERROR","message":"Database connection failed","metadata":{"error_code":500,"service":"auth"}}"#;
        
        let entry = parse_log_line(json_line).unwrap();
        
        assert_eq!(entry.timestamp, "2024-01-15T10:30:00Z");
        assert_eq!(entry.level, "ERROR");
        assert_eq!(entry.message, "Database connection failed");
        assert_eq!(entry.metadata["error_code"], 500);
    }

    #[test]
    fn test_parse_invalid_json() {
        let invalid_line = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"ERROR""#;
        
        assert!(parse_log_line(invalid_line).is_err());
    }

    #[test]
    fn test_filter_logs() {
        let entries = vec![
            LogEntry {
                timestamp: "2024-01-15T10:30:00Z".to_string(),
                level: "ERROR".to_string(),
                message: "Test error".to_string(),
                metadata: Value::Null,
            },
            LogEntry {
                timestamp: "2024-01-15T10:31:00Z".to_string(),
                level: "INFO".to_string(),
                message: "Test info".to_string(),
                metadata: Value::Null,
            },
        ];
        
        let errors = filter_logs_by_level(&entries, "ERROR");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].level, "ERROR");
    }
}