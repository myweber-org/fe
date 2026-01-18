
use serde_json::{Value, Error as JsonError};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub fields: HashMap<String, Value>,
}

#[derive(Debug)]
pub enum LogError {
    IoError(std::io::Error),
    JsonError(JsonError),
    InvalidFormat(String),
}

impl From<std::io::Error> for LogError {
    fn from(err: std::io::Error) -> Self {
        LogError::IoError(err)
    }
}

impl From<JsonError> for LogError {
    fn from(err: JsonError) -> Self {
        LogError::JsonError(err)
    }
}

pub fn parse_log_line(line: &str) -> Result<LogEntry, LogError> {
    let json_value: Value = serde_json::from_str(line)?;
    
    let obj = json_value.as_object()
        .ok_or_else(|| LogError::InvalidFormat("Expected JSON object".to_string()))?;
    
    let timestamp = obj.get("timestamp")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LogError::InvalidFormat("Missing timestamp field".to_string()))?
        .to_string();
    
    let level = obj.get("level")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LogError::InvalidFormat("Missing level field".to_string()))?
        .to_string();
    
    let message = obj.get("message")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LogError::InvalidFormat("Missing message field".to_string()))?
        .to_string();
    
    let mut fields = HashMap::new();
    for (key, value) in obj {
        if !["timestamp", "level", "message"].contains(&key.as_str()) {
            fields.insert(key.clone(), value.clone());
        }
    }
    
    Ok(LogEntry {
        timestamp,
        level,
        message,
        fields,
    })
}

pub fn process_log_file<P: AsRef<Path>>(path: P) -> Result<Vec<LogEntry>, LogError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();
    
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        
        match parse_log_line(&line) {
            Ok(entry) => entries.push(entry),
            Err(e) => eprintln!("Failed to parse line: {:?}", e),
        }
    }
    
    Ok(entries)
}

pub fn filter_by_level(entries: &[LogEntry], level: &str) -> Vec<&LogEntry> {
    entries.iter()
        .filter(|entry| entry.level.eq_ignore_ascii_case(level))
        .collect()
}

pub fn extract_field_values(entries: &[LogEntry], field_name: &str) -> Vec<&Value> {
    entries.iter()
        .filter_map(|entry| entry.fields.get(field_name))
        .collect()
}