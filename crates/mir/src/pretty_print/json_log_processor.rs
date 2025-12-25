use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    service: String,
    message: String,
    metadata: HashMap<String, Value>,
}

#[derive(Error, Debug)]
enum LogError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Invalid log level: {0}")]
    InvalidLevel(String),
    #[error("Missing required field: {0}")]
    MissingField(String),
}

impl LogEntry {
    fn from_json_line(line: &str) -> Result<Self, LogError> {
        let mut entry: LogEntry = serde_json::from_str(line)?;
        
        let valid_levels = ["ERROR", "WARN", "INFO", "DEBUG", "TRACE"];
        if !valid_levels.contains(&entry.level.as_str()) {
            return Err(LogError::InvalidLevel(entry.level.clone()));
        }
        
        if entry.timestamp.is_empty() || entry.service.is_empty() {
            return Err(LogError::MissingField("timestamp or service".to_string()));
        }
        
        Ok(entry)
    }
    
    fn is_error(&self) -> bool {
        self.level == "ERROR"
    }
    
    fn contains_keyword(&self, keyword: &str) -> bool {
        self.message.contains(keyword) || 
        self.metadata.values().any(|v| {
            v.as_str().map(|s| s.contains(keyword)).unwrap_or(false)
        })
    }
}

struct LogProcessor {
    entries: Vec<LogEntry>,
    error_count: usize,
}

impl LogProcessor {
    fn new() -> Self {
        LogProcessor {
            entries: Vec::new(),
            error_count: 0,
        }
    }
    
    fn load_from_file(&mut self, path: &str) -> Result<(), LogError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            match LogEntry::from_json_line(&line) {
                Ok(entry) => {
                    if entry.is_error() {
                        self.error_count += 1;
                    }
                    self.entries.push(entry);
                }
                Err(e) => {
                    eprintln!("Line {}: {}", line_num + 1, e);
                }
            }
        }
        
        Ok(())
    }
    
    fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }
    
    fn search_entries(&self, keyword: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.contains_keyword(keyword))
            .collect()
    }
    
    fn get_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        
        for entry in &self.entries {
            *stats.entry(entry.level.clone()).or_insert(0) += 1;
            *stats.entry(entry.service.clone()).or_insert(0) += 1;
        }
        
        stats.insert("total".to_string(), self.entries.len());
        stats.insert("errors".to_string(), self.error_count);
        
        stats
    }
}

fn process_log_file(path: &str) -> Result<(), LogError> {
    let mut processor = LogProcessor::new();
    processor.load_from_file(path)?;
    
    let stats = processor.get_stats();
    println!("Log Statistics:");
    for (key, value) in stats {
        println!("  {}: {}", key, value);
    }
    
    let errors = processor.filter_by_level("ERROR");
    if !errors.is_empty() {
        println!("\nError entries found:");
        for error in errors {
            println!("  [{}] {}: {}", error.timestamp, error.service, error.message);
        }
    }
    
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <log_file.json>", args[0]);
        std::process::exit(1);
    }
    
    if let Err(e) = process_log_file(&args[1]) {
        eprintln!("Failed to process log file: {}", e);
        std::process::exit(1);
    }
}