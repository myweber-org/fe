
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Deserialize, Serialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    service: String,
    message: String,
    metadata: Option<serde_json::Value>,
}

#[derive(Debug)]
enum LogError {
    IoError(std::io::Error),
    ParseError(serde_json::Error),
    ValidationError(String),
}

impl fmt::Display for LogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogError::IoError(e) => write!(f, "IO error: {}", e),
            LogError::ParseError(e) => write!(f, "Parse error: {}", e),
            LogError::ValidationError(e) => write!(f, "Validation error: {}", e),
        }
    }
}

impl Error for LogError {}

impl From<std::io::Error> for LogError {
    fn from(error: std::io::Error) -> Self {
        LogError::IoError(error)
    }
}

impl From<serde_json::Error> for LogError {
    fn from(error: serde_json::Error) -> Self {
        LogError::ParseError(error)
    }
}

impl LogEntry {
    fn validate(&self) -> Result<(), LogError> {
        if self.timestamp.is_empty() {
            return Err(LogError::ValidationError("Timestamp cannot be empty".to_string()));
        }
        
        let valid_levels = ["INFO", "WARN", "ERROR", "DEBUG"];
        if !valid_levels.contains(&self.level.as_str()) {
            return Err(LogError::ValidationError(
                format!("Invalid log level: {}", self.level)
            ));
        }
        
        if self.service.is_empty() {
            return Err(LogError::ValidationError("Service name cannot be empty".to_string()));
        }
        
        Ok(())
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
    
    fn process_file(&mut self, path: &str) -> Result<(), LogError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            match self.parse_line(&line) {
                Ok(entry) => {
                    if let Err(e) = entry.validate() {
                        eprintln!("Line {}: {}", line_num + 1, e);
                        self.error_count += 1;
                    } else {
                        self.entries.push(entry);
                    }
                }
                Err(e) => {
                    eprintln!("Line {}: {}", line_num + 1, e);
                    self.error_count += 1;
                }
            }
        }
        
        Ok(())
    }
    
    fn parse_line(&self, line: &str) -> Result<LogEntry, LogError> {
        let entry: LogEntry = serde_json::from_str(line)?;
        Ok(entry)
    }
    
    fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }
    
    fn get_stats(&self) -> (usize, usize) {
        (self.entries.len(), self.error_count)
    }
    
    fn export_json(&self, path: &str) -> Result<(), LogError> {
        let file = File::create(path)?;
        serde_json::to_writer_pretty(file, &self.entries)?;
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut processor = LogProcessor::new();
    
    match processor.process_file("logs.jsonl") {
        Ok(_) => {
            let (processed, errors) = processor.get_stats();
            println!("Processed {} entries, encountered {} errors", processed, errors);
            
            let error_logs = processor.filter_by_level("ERROR");
            println!("Found {} ERROR level logs", error_logs.len());
            
            if !error_logs.is_empty() {
                for log in error_logs.iter().take(3) {
                    println!("  - {}: {}", log.timestamp, log.message);
                }
            }
            
            processor.export_json("processed_logs.json")?;
        }
        Err(e) => {
            eprintln!("Failed to process logs: {}", e);
            return Err(Box::new(e));
        }
    }
    
    Ok(())
}