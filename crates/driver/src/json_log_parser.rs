use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub fields: HashMap<String, Value>,
}

pub struct LogParser {
    min_level: Option<String>,
    filter_fields: HashMap<String, Value>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            min_level: None,
            filter_fields: HashMap::new(),
        }
    }

    pub fn set_min_level(&mut self, level: &str) {
        self.min_level = Some(level.to_lowercase());
    }

    pub fn add_filter(&mut self, key: &str, value: Value) {
        self.filter_fields.insert(key.to_string(), value);
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
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

    pub fn parse_line(&self, line: &str) -> Result<LogEntry, Box<dyn std::error::Error>> {
        let json_value: Value = serde_json::from_str(line)?;
        
        let mut fields = HashMap::new();
        if let Value::Object(map) = json_value {
            for (key, value) in map {
                fields.insert(key, value);
            }
        }

        let timestamp = fields
            .get("timestamp")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let level = fields
            .get("level")
            .and_then(|v| v.as_str())
            .unwrap_or("info")
            .to_lowercase();

        let message = fields
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let entry = LogEntry {
            timestamp,
            level,
            message,
            fields,
        };

        if !self.passes_filters(&entry) {
            return Err("Entry filtered out".into());
        }

        Ok(entry)
    }

    fn passes_filters(&self, entry: &LogEntry) -> bool {
        if let Some(min_level) = &self.min_level {
            let level_order = |lvl: &str| match lvl {
                "trace" => 0,
                "debug" => 1,
                "info" => 2,
                "warn" => 3,
                "error" => 4,
                _ => 5,
            };

            if level_order(&entry.level) < level_order(min_level) {
                return false;
            }
        }

        for (key, filter_value) in &self.filter_fields {
            if let Some(entry_value) = entry.fields.get(key) {
                if entry_value != filter_value {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }

    pub fn format_entry(&self, entry: &LogEntry, format: &str) -> String {
        match format {
            "json" => serde_json::to_string_pretty(entry).unwrap_or_default(),
            "simple" => format!(
                "[{}] {}: {}",
                entry.timestamp, entry.level.to_uppercase(), entry.message
            ),
            "detailed" => {
                let mut output = format!(
                    "Timestamp: {}\nLevel: {}\nMessage: {}\n",
                    entry.timestamp, entry.level, entry.message
                );
                for (key, value) in &entry.fields {
                    if key != "timestamp" && key != "level" && key != "message" {
                        output.push_str(&format!("{}: {}\n", key, value));
                    }
                }
                output
            }
            _ => format!("{:?}", entry),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_valid_line() {
        let parser = LogParser::new();
        let line = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"info","message":"Service started","service":"api"}"#;
        
        let result = parser.parse_line(line);
        assert!(result.is_ok());
        
        let entry = result.unwrap();
        assert_eq!(entry.timestamp, "2024-01-15T10:30:00Z");
        assert_eq!(entry.level, "info");
        assert_eq!(entry.message, "Service started");
        assert_eq!(entry.fields.get("service").unwrap(), &json!("api"));
    }

    #[test]
    fn test_level_filtering() {
        let mut parser = LogParser::new();
        parser.set_min_level("warn");
        
        let warn_line = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"warn","message":"High memory usage"}"#;
        let info_line = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"info","message":"Service started"}"#;
        
        assert!(parser.parse_line(warn_line).is_ok());
        assert!(parser.parse_line(info_line).is_err());
    }

    #[test]
    fn test_field_filtering() {
        let mut parser = LogParser::new();
        parser.add_filter("service", json!("api"));
        
        let matching_line = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"info","message":"Request processed","service":"api"}"#;
        let non_matching_line = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"info","message":"Request processed","service":"db"}"#;
        
        assert!(parser.parse_line(matching_line).is_ok());
        assert!(parser.parse_line(non_matching_line).is_err());
    }
}use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    service: String,
    message: String,
    metadata: HashMap<String, String>,
}

struct LogParser {
    entries: Vec<LogEntry>,
}

impl LogParser {
    fn new() -> Self {
        LogParser {
            entries: Vec::new(),
        }
    }

    fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<LogEntry>(&line) {
                Ok(entry) => self.entries.push(entry),
                Err(e) => eprintln!("Failed to parse line: {}", e),
            }
        }

        Ok(())
    }

    fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .collect()
    }

    fn filter_by_service(&self, service: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.service == service)
            .collect()
    }

    fn count_by_level(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for entry in &self.entries {
            *counts.entry(entry.level.clone()).or_insert(0) += 1;
        }
        counts
    }

    fn get_unique_services(&self) -> Vec<String> {
        let mut services: Vec<String> = self.entries.iter().map(|e| e.service.clone()).collect();
        services.sort();
        services.dedup();
        services
    }

    fn search_in_messages(&self, query: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.message.to_lowercase().contains(&query.to_lowercase()))
            .collect()
    }

    fn get_time_range(&self) -> Option<(String, String)> {
        if self.entries.is_empty() {
            return None;
        }

        let mut timestamps: Vec<&String> = self.entries.iter().map(|e| &e.timestamp).collect();
        timestamps.sort();

        Some((timestamps[0].clone(), timestamps[timestamps.len() - 1].clone()))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = LogParser::new();
    
    parser.load_from_file("logs.jsonl")?;
    
    println!("Total log entries: {}", parser.entries.len());
    
    let error_logs = parser.filter_by_level("ERROR");
    println!("Error logs: {}", error_logs.len());
    
    let level_counts = parser.count_by_level();
    println!("Log level distribution: {:?}", level_counts);
    
    let services = parser.get_unique_services();
    println!("Unique services: {:?}", services);
    
    if let Some((start, end)) = parser.get_time_range() {
        println!("Time range: {} - {}", start, end);
    }
    
    let search_results = parser.search_in_messages("timeout");
    println!("Found {} entries containing 'timeout'", search_results.len());
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let log_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","service":"api","message":"Request processed","metadata":{"user_id":"123"}}
{"timestamp":"2024-01-15T10:31:00Z","level":"ERROR","service":"db","message":"Connection timeout","metadata":{"retry_count":"3"}}
{"timestamp":"2024-01-15T10:32:00Z","level":"WARN","service":"cache","message":"Memory usage high","metadata":{"usage":"85%"}}"#;
        
        writeln!(temp_file, "{}", log_data).unwrap();
        
        let mut parser = LogParser::new();
        parser.load_from_file(temp_file.path()).unwrap();
        
        assert_eq!(parser.entries.len(), 3);
        assert_eq!(parser.filter_by_level("ERROR").len(), 1);
        assert_eq!(parser.filter_by_service("api").len(), 1);
        assert_eq!(parser.search_in_messages("timeout").len(), 1);
        
        let counts = parser.count_by_level();
        assert_eq!(counts.get("INFO"), Some(&1));
        assert_eq!(counts.get("ERROR"), Some(&1));
        assert_eq!(counts.get("WARN"), Some(&1));
    }
}