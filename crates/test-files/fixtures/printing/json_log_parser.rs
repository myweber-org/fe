
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub service: String,
    pub message: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug)]
pub enum ParseError {
    IoError(String),
    JsonError(serde_json::Error),
    ValidationError(String),
    InvalidTimestamp(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::IoError(msg) => write!(f, "IO error: {}", msg),
            ParseError::JsonError(err) => write!(f, "JSON parsing error: {}", err),
            ParseError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            ParseError::InvalidTimestamp(msg) => write!(f, "Invalid timestamp: {}", msg),
        }
    }
}

impl Error for ParseError {}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::IoError(err.to_string())
    }
}

impl From<serde_json::Error> for ParseError {
    fn from(err: serde_json::Error) -> Self {
        ParseError::JsonError(err)
    }
}

pub struct LogParser {
    min_log_level: String,
    service_filter: Option<String>,
}

impl LogParser {
    pub fn new(min_log_level: &str) -> Self {
        LogParser {
            min_log_level: min_log_level.to_lowercase(),
            service_filter: None,
        }
    }

    pub fn with_service_filter(mut self, service: &str) -> Self {
        self.service_filter = Some(service.to_string());
        self
    }

    pub fn parse_file(&self, file_path: &str) -> Result<Vec<LogEntry>, ParseError> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        let mut entries = Vec::new();
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            match self.parse_line(&line) {
                Ok(Some(entry)) => entries.push(entry),
                Ok(None) => continue,
                Err(err) => eprintln!("Warning: Failed to parse line {}: {}", line_num + 1, err),
            }
        }
        
        Ok(entries)
    }

    fn parse_line(&self, line: &str) -> Result<Option<LogEntry>, ParseError> {
        let raw_entry: serde_json::Value = serde_json::from_str(line)?;
        
        let level = raw_entry.get("level")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ParseError::ValidationError("Missing 'level' field".to_string()))?
            .to_lowercase();
        
        if !self.is_level_allowed(&level) {
            return Ok(None);
        }

        let service = raw_entry.get("service")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ParseError::ValidationError("Missing 'service' field".to_string()))?
            .to_string();
        
        if let Some(ref filter) = self.service_filter {
            if &service != filter {
                return Ok(None);
            }
        }

        let timestamp_str = raw_entry.get("timestamp")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ParseError::ValidationError("Missing 'timestamp' field".to_string()))?;
        
        let timestamp = DateTime::parse_from_rfc3339(timestamp_str)
            .map_err(|_| ParseError::InvalidTimestamp(timestamp_str.to_string()))?
            .with_timezone(&Utc);

        let message = raw_entry.get("message")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ParseError::ValidationError("Missing 'message' field".to_string()))?
            .to_string();

        let metadata = raw_entry.get("metadata")
            .cloned()
            .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));

        Ok(Some(LogEntry {
            timestamp,
            level,
            service,
            message,
            metadata,
        }))
    }

    fn is_level_allowed(&self, level: &str) -> bool {
        let level_order = ["trace", "debug", "info", "warn", "error", "critical"];
        
        let min_index = level_order.iter()
            .position(|&l| l == self.min_log_level)
            .unwrap_or(0);
        
        let entry_index = level_order.iter()
            .position(|&l| l == level)
            .unwrap_or(level_order.len());
        
        entry_index >= min_index
    }
}

pub fn analyze_logs(entries: &[LogEntry]) -> (usize, String, String) {
    let total = entries.len();
    
    let error_count = entries.iter()
        .filter(|e| e.level == "error" || e.level == "critical")
        .count();
    
    let latest_timestamp = entries.iter()
        .map(|e| e.timestamp)
        .max()
        .map(|ts| ts.to_rfc3339())
        .unwrap_or_else(|| "No entries".to_string());
    
    let services: Vec<String> = entries.iter()
        .map(|e| e.service.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    
    (error_count, latest_timestamp, services.join(", "))
}