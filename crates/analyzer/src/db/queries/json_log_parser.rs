use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    service: String,
    #[serde(default)]
    metadata: serde_json::Value,
}

#[derive(Debug)]
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

    fn parse_file(&self, path: &Path) -> Result<Vec<LogEntry>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            match self.parse_line(&line) {
                Ok(Some(entry)) => entries.push(entry),
                Ok(None) => continue,
                Err(e) => eprintln!("Failed to parse line: {}\nError: {}", line, e),
            }
        }

        Ok(entries)
    }

    fn parse_line(&self, line: &str) -> Result<Option<LogEntry>, Box<dyn Error>> {
        let entry: LogEntry = serde_json::from_str(line)?;

        if !self.matches_level(&entry.level) {
            return Ok(None);
        }

        if let Some(ref service) = self.service_filter {
            if entry.service != *service {
                return Ok(None);
            }
        }

        Ok(Some(entry))
    }

    fn matches_level(&self, level: &str) -> bool {
        let level_order = ["trace", "debug", "info", "warn", "error", "fatal"];
        let entry_level = level.to_lowercase();
        let min_index = level_order.iter().position(|&l| l == self.min_level);
        let entry_index = level_order.iter().position(|&l| l == entry_level);

        match (min_index, entry_index) {
            (Some(min), Some(entry)) => entry >= min,
            _ => false,
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let parser = LogParser::new("info")
        .with_service_filter("api-service");

    let entries = parser.parse_file(Path::new("logs/app.log"))?;

    println!("Found {} log entries", entries.len());
    
    for entry in entries.iter().take(5) {
        println!("{:?}", entry);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_log() {
        let log_data = r#"{"timestamp": "2024-01-15T10:30:00Z", "level": "INFO", "message": "Service started", "service": "api-service"}"#;
        let parser = LogParser::new("info");
        let result = parser.parse_line(log_data).unwrap();
        assert!(result.is_some());
        let entry = result.unwrap();
        assert_eq!(entry.level, "INFO");
        assert_eq!(entry.service, "api-service");
    }

    #[test]
    fn test_level_filtering() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{"timestamp": "2024-01-15T10:30:00Z", "level": "DEBUG", "message": "Debug message", "service": "api-service"}}"#).unwrap();
        writeln!(temp_file, r#"{{"timestamp": "2024-01-15T10:31:00Z", "level": "ERROR", "message": "Error occurred", "service": "api-service"}}"#).unwrap();

        let parser = LogParser::new("warn");
        let entries = parser.parse_file(temp_file.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].level, "ERROR");
    }
}