
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
    metadata: Option<serde_json::Value>,
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
    entries: Vec<LogEntry>,
}

impl LogProcessor {
    pub fn new() -> Self {
        LogProcessor {
            entries: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), LogError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for (line_num, line) in reader.lines().enumerate() {
            let line_content = line?;
            match self.parse_line(&line_content) {
                Ok(entry) => self.entries.push(entry),
                Err(e) => eprintln!("Warning: Failed to parse line {}: {:?}", line_num + 1, e),
            }
        }

        Ok(())
    }

    pub fn parse_line(&self, line: &str) -> Result<LogEntry, LogError> {
        let parsed: serde_json::Value = serde_json::from_str(line)?;

        let timestamp = parsed["timestamp"]
            .as_str()
            .ok_or_else(|| LogError::InvalidFormat("Missing timestamp field".to_string()))?
            .to_string();

        let level = parsed["level"]
            .as_str()
            .ok_or_else(|| LogError::InvalidFormat("Missing level field".to_string()))?
            .to_string();

        let message = parsed["message"]
            .as_str()
            .ok_or_else(|| LogError::InvalidFormat("Missing message field".to_string()))?
            .to_string();

        let metadata = parsed.get("metadata").cloned();

        Ok(LogEntry {
            timestamp,
            level,
            message,
            metadata,
        })
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .collect()
    }

    pub fn count_entries(&self) -> usize {
        self.entries.len()
    }

    pub fn export_to_json(&self) -> Result<String, LogError> {
        let json = serde_json::to_string_pretty(&self.entries)?;
        Ok(json)
    }
}

impl Default for LogProcessor {
    fn default() -> Self {
        Self::new()
    }
}