
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum LogLevel {
    DEBUG,
    INFO,
    WARN,
    ERROR,
    CRITICAL,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub message: String,
    pub metadata: HashMap<String, String>,
}

pub struct LogProcessor {
    min_level: LogLevel,
    filters: Vec<String>,
}

impl LogProcessor {
    pub fn new(min_level: LogLevel) -> Self {
        LogProcessor {
            min_level,
            filters: Vec::new(),
        }
    }

    pub fn add_filter(&mut self, keyword: &str) {
        self.filters.push(keyword.to_lowercase());
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line_content = line.map_err(|e| format!("Line {} read error: {}", line_num + 1, e))?;
            
            match serde_json::from_str::<LogEntry>(&line_content) {
                Ok(mut entry) => {
                    if self.should_process(&entry) {
                        self.normalize_metadata(&mut entry);
                        entries.push(entry);
                    }
                }
                Err(e) => eprintln!("Line {} parse error: {} - Content: {}", line_num + 1, e, line_content),
            }
        }

        Ok(entries)
    }

    fn should_process(&self, entry: &LogEntry) -> bool {
        if entry.level < self.min_level {
            return false;
        }

        if self.filters.is_empty() {
            return true;
        }

        let message_lower = entry.message.to_lowercase();
        self.filters.iter().any(|filter| message_lower.contains(filter))
    }

    fn normalize_metadata(&self, entry: &mut LogEntry) {
        entry.metadata.retain(|_, v| !v.trim().is_empty());
    }

    pub fn export_to_json(&self, entries: &[LogEntry], output_path: &str) -> Result<(), String> {
        let json = serde_json::to_string_pretty(entries)
            .map_err(|e| format!("Failed to serialize entries: {}", e))?;
        
        std::fs::write(output_path, json)
            .map_err(|e| format!("Failed to write output file: {}", e))?;
        
        Ok(())
    }
}

impl PartialOrd for LogLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let self_val = match self {
            LogLevel::DEBUG => 0,
            LogLevel::INFO => 1,
            LogLevel::WARN => 2,
            LogLevel::ERROR => 3,
            LogLevel::CRITICAL => 4,
        };
        let other_val = match other {
            LogLevel::DEBUG => 0,
            LogLevel::INFO => 1,
            LogLevel::WARN => 2,
            LogLevel::ERROR => 3,
            LogLevel::CRITICAL => 4,
        };
        Some(self_val.cmp(&other_val))
    }
}

impl Ord for LogLevel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}