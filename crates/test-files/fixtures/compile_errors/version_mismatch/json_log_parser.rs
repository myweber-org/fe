use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize, Serialize)]
struct LogEntry {
    timestamp: DateTime<Utc>,
    level: String,
    message: String,
    service: String,
    #[serde(default)]
    metadata: serde_json::Value,
}

struct LogParser {
    min_level: String,
    service_filter: Option<String>,
}

impl LogParser {
    fn new(min_level: &str) -> Self {
        LogParser {
            min_level: min_level.to_lowercase(),
            service_filter: None,
        }
    }

    fn with_service_filter(mut self, service: &str) -> Self {
        self.service_filter = Some(service.to_string());
        self
    }

    fn parse_file(&self, file_path: &str) -> Result<Vec<LogEntry>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = serde_json::from_str::<LogEntry>(&line) {
                if self.matches_filter(&entry) {
                    entries.push(entry);
                }
            }
        }

        Ok(entries)
    }

    fn matches_filter(&self, entry: &LogEntry) -> bool {
        let level_priority = |level: &str| match level.to_lowercase().as_str() {
            "error" => 4,
            "warn" => 3,
            "info" => 2,
            "debug" => 1,
            _ => 0,
        };

        let entry_priority = level_priority(&entry.level);
        let min_priority = level_priority(&self.min_level);

        if entry_priority < min_priority {
            return false;
        }

        if let Some(ref service) = self.service_filter {
            if entry.service != *service {
                return false;
            }
        }

        true
    }

    fn count_by_level(&self, entries: &[LogEntry]) -> std::collections::HashMap<String, usize> {
        let mut counts = std::collections::HashMap::new();
        
        for entry in entries {
            *counts.entry(entry.level.clone()).or_insert(0) += 1;
        }
        
        counts
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let parser = LogParser::new("info")
        .with_service_filter("api-service");

    let entries = parser.parse_file("logs/app.log")?;
    
    println!("Found {} log entries", entries.len());
    
    let counts = parser.count_by_level(&entries);
    for (level, count) in counts {
        println!("{}: {}", level, count);
    }

    if let Some(error_entry) = entries.iter().find(|e| e.level == "error") {
        println!("\nLatest error: {}", error_entry.message);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_log_parser_filtering() {
        let test_entry = LogEntry {
            timestamp: Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap(),
            level: "ERROR".to_string(),
            message: "Test error".to_string(),
            service: "api-service".to_string(),
            metadata: serde_json::json!({}),
        };

        let parser = LogParser::new("warn");
        assert!(parser.matches_filter(&test_entry));

        let parser_low = LogParser::new("error");
        assert!(parser_low.matches_filter(&test_entry));

        let parser_high = LogParser::new("fatal");
        assert!(!parser_high.matches_filter(&test_entry));
    }
}