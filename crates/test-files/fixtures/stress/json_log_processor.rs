use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

#[derive(Debug, Deserialize, Serialize)]
struct LogEntry {
    timestamp: String,
    level: LogLevel,
    module: String,
    message: String,
    metadata: HashMap<String, String>,
}

struct LogProcessor {
    min_level: LogLevel,
    module_filter: Option<String>,
}

impl LogProcessor {
    fn new(min_level: LogLevel) -> Self {
        LogProcessor {
            min_level,
            module_filter: None,
        }
    }

    fn with_module_filter(mut self, module: &str) -> Self {
        self.module_filter = Some(module.to_string());
        self
    }

    fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut filtered_entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = serde_json::from_str::<LogEntry>(&line) {
                if self.should_include(&entry) {
                    filtered_entries.push(entry);
                }
            }
        }

        Ok(filtered_entries)
    }

    fn should_include(&self, entry: &LogEntry) -> bool {
        if entry.level < self.min_level {
            return false;
        }

        if let Some(ref module_filter) = self.module_filter {
            if !entry.module.contains(module_filter) {
                return false;
            }
        }

        true
    }

    fn generate_summary(&self, entries: &[LogEntry]) -> HashMap<LogLevel, usize> {
        let mut summary = HashMap::new();
        for entry in entries {
            *summary.entry(entry.level.clone()).or_insert(0) += 1;
        }
        summary
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let processor = LogProcessor::new(LogLevel::Info)
        .with_module_filter("api");

    let entries = processor.process_file("logs/app.log")?;
    
    println!("Found {} relevant log entries", entries.len());
    
    let summary = processor.generate_summary(&entries);
    for (level, count) in summary {
        println!("{:?}: {}", level, count);
    }

    if let Some(error_entry) = entries.iter().find(|e| e.level == LogLevel::Error) {
        println!("Latest error: {} - {}", error_entry.timestamp, error_entry.message);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Debug > LogLevel::Trace);
        assert!(LogLevel::Info > LogLevel::Debug);
        assert!(LogLevel::Error > LogLevel::Warn);
    }

    #[test]
    fn test_processor_filtering() {
        let processor = LogProcessor::new(LogLevel::Warn);
        let test_entry = LogEntry {
            timestamp: "2024-01-15T10:30:00Z".to_string(),
            level: LogLevel::Info,
            module: "database".to_string(),
            message: "Connection established".to_string(),
            metadata: HashMap::new(),
        };

        assert!(!processor.should_include(&test_entry));
    }
}