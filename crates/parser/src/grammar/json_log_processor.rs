use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    #[serde(default)]
    metadata: serde_json::Value,
}

#[derive(Debug)]
pub enum LogError {
    IoError(std::io::Error),
    ParseError(serde_json::Error),
    InvalidFormat(String),
}

impl From<std::io::Error> for LogError {
    fn from(err: std::io::Error) -> Self {
        LogError::IoError(err)
    }
}

impl From<serde_json::Error> for LogError {
    fn from(err: serde_json::Error) -> Self {
        LogError::ParseError(err)
    }
}

pub struct LogProcessor {
    pub entries: Vec<LogEntry>,
}

impl LogProcessor {
    pub fn new() -> Self {
        LogProcessor {
            entries: Vec::new(),
        }
    }

    pub fn process_file<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, LogError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut count = 0;
        for line_result in reader.lines() {
            let line = line_result?;
            if line.trim().is_empty() {
                continue;
            }
            
            let entry: LogEntry = serde_json::from_str(&line)?;
            self.entries.push(entry);
            count += 1;
        }
        
        Ok(count)
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.eq_ignore_ascii_case(level))
            .collect()
    }

    pub fn get_stats(&self) -> std::collections::HashMap<String, usize> {
        let mut stats = std::collections::HashMap::new();
        
        for entry in &self.entries {
            *stats.entry(entry.level.clone()).or_insert(0) += 1;
        }
        
        stats
    }
}

pub fn validate_log_format(json_str: &str) -> Result<LogEntry, LogError> {
    let entry: LogEntry = serde_json::from_str(json_str)?;
    
    if entry.timestamp.is_empty() || entry.level.is_empty() || entry.message.is_empty() {
        return Err(LogError::InvalidFormat(
            "Missing required fields: timestamp, level, or message".to_string()
        ));
    }
    
    Ok(entry)
}