use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum LogLevel {
    ERROR,
    WARN,
    INFO,
    DEBUG,
    TRACE,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub component: String,
    pub metadata: Option<serde_json::Value>,
}

pub struct LogFilter {
    pub min_level: Option<LogLevel>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub component_filter: Option<String>,
}

impl LogFilter {
    pub fn matches(&self, entry: &LogEntry) -> bool {
        if let Some(min_level) = &self.min_level {
            if !Self::level_at_least(&entry.level, min_level) {
                return false;
            }
        }

        if let Some(start) = &self.start_time {
            if &entry.timestamp < start {
                return false;
            }
        }

        if let Some(end) = &self.end_time {
            if &entry.timestamp > end {
                return false;
            }
        }

        if let Some(component) = &self.component_filter {
            if !entry.component.contains(component) {
                return false;
            }
        }

        true
    }

    fn level_at_least(entry_level: &LogLevel, min_level: &LogLevel) -> bool {
        match (entry_level, min_level) {
            (LogLevel::ERROR, _) => true,
            (LogLevel::WARN, LogLevel::ERROR) => false,
            (LogLevel::WARN, _) => true,
            (LogLevel::INFO, LogLevel::ERROR | LogLevel::WARN) => false,
            (LogLevel::INFO, _) => true,
            (LogLevel::DEBUG, LogLevel::TRACE) => false,
            (LogLevel::DEBUG, _) => true,
            (LogLevel::TRACE, LogLevel::TRACE) => true,
            (LogLevel::TRACE, _) => false,
        }
    }
}

pub struct LogParser {
    filter: LogFilter,
}

impl LogParser {
    pub fn new(filter: LogFilter) -> Self {
        LogParser { filter }
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<LogEntry>(&line) {
                Ok(entry) => {
                    if self.filter.matches(&entry) {
                        entries.push(entry);
                    }
                }
                Err(e) => eprintln!("Failed to parse line: {}. Error: {}", line, e),
            }
        }

        Ok(entries)
    }

    pub fn count_by_level(&self, entries: &[LogEntry]) -> std::collections::HashMap<LogLevel, usize> {
        let mut counts = std::collections::HashMap::new();
        
        for entry in entries {
            *counts.entry(entry.level.clone()).or_insert(0) += 1;
        }
        
        counts
    }
}

pub fn export_to_jsonl(entries: &[LogEntry], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(output_path)?;
    let mut writer = std::io::BufWriter::new(file);

    for entry in entries {
        let json = serde_json::to_string(entry)?;
        writeln!(writer, "{}", json)?;
    }

    Ok(())
}