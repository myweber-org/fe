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
            if let Some(metadata_value) = json_value.get(key) {
                if metadata_value.as_str() != Some(value) {
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

pub fn print_entries(entries: &[LogEntry]) {
    for entry in entries {
        println!(
            "[{}] {}: {}",
            entry.timestamp, entry.level.to_uppercase(), entry.message
        );
        if !entry.metadata.is_null() {
            println!("Metadata: {}", entry.metadata);
        }
        println!("---");
    }
}