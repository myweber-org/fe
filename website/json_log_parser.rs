
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use chrono::{DateTime, FixedOffset};
use serde_json::Value;

#[derive(Debug)]
pub enum ParseError {
    IoError(io::Error),
    JsonError(serde_json::Error),
    InvalidTimestamp,
}

impl From<io::Error> for ParseError {
    fn from(err: io::Error) -> Self {
        ParseError::IoError(err)
    }
}

impl From<serde_json::Error> for ParseError {
    fn from(err: serde_json::Error) -> Self {
        ParseError::JsonError(err)
    }
}

pub struct LogEntry {
    pub timestamp: DateTime<FixedOffset>,
    pub level: String,
    pub message: String,
    pub metadata: Value,
}

pub struct LogParser {
    path: String,
    filter_level: Option<String>,
    start_time: Option<DateTime<FixedOffset>>,
    end_time: Option<DateTime<FixedOffset>>,
}

impl LogParser {
    pub fn new(path: &str) -> Self {
        LogParser {
            path: path.to_string(),
            filter_level: None,
            start_time: None,
            end_time: None,
        }
    }

    pub fn with_level_filter(mut self, level: &str) -> Self {
        self.filter_level = Some(level.to_string());
        self
    }

    pub fn with_time_range(
        mut self,
        start: DateTime<FixedOffset>,
        end: DateTime<FixedOffset>,
    ) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
    }

    pub fn parse(&self) -> Result<Vec<LogEntry>, ParseError> {
        let file = File::open(Path::new(&self.path))?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let json_value: Value = serde_json::from_str(&line)?;
            
            let timestamp_str = json_value["timestamp"]
                .as_str()
                .ok_or(ParseError::InvalidTimestamp)?;
            
            let timestamp = DateTime::parse_from_rfc3339(timestamp_str)
                .map_err(|_| ParseError::InvalidTimestamp)?;

            if let Some(ref start) = self.start_time {
                if timestamp < *start {
                    continue;
                }
            }

            if let Some(ref end) = self.end_time {
                if timestamp > *end {
                    continue;
                }
            }

            let level = json_value["level"]
                .as_str()
                .unwrap_or("UNKNOWN")
                .to_string();

            if let Some(ref filter_level) = self.filter_level {
                if level.to_uppercase() != filter_level.to_uppercase() {
                    continue;
                }
            }

            let message = json_value["message"]
                .as_str()
                .unwrap_or("")
                .to_string();

            let metadata = json_value["metadata"].clone();

            entries.push(LogEntry {
                timestamp,
                level,
                message,
                metadata,
            });
        }

        Ok(entries)
    }

    pub fn count_entries(&self) -> Result<usize, ParseError> {
        self.parse().map(|entries| entries.len())
    }

    pub fn get_error_entries(&self) -> Result<Vec<LogEntry>, ParseError> {
        let parser = LogParser::new(&self.path)
            .with_level_filter("ERROR")
            .with_time_range(
                self.start_time.unwrap_or(DateTime::parse_from_rfc3339("1970-01-01T00:00:00Z").unwrap()),
                self.end_time.unwrap_or(DateTime::parse_from_rfc3339("2100-01-01T00:00:00Z").unwrap()),
            );
        parser.parse()
    }
}

pub fn analyze_log_file(path: &str) -> Result<(), ParseError> {
    let parser = LogParser::new(path);
    let entries = parser.parse()?;
    
    let error_count = parser.get_error_entries()?.len();
    let total_count = entries.len();
    
    if total_count > 0 {
        let error_percentage = (error_count as f64 / total_count as f64) * 100.0;
        println!("Total entries: {}", total_count);
        println!("Error entries: {}", error_count);
        println!("Error percentage: {:.2}%", error_percentage);
        
        if let Some(first) = entries.first() {
            println!("First entry: {}", first.timestamp);
        }
        if let Some(last) = entries.last() {
            println!("Last entry: {}", last.timestamp);
        }
    } else {
        println!("No log entries found");
    }
    
    Ok(())
}