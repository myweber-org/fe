use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    service: String,
    message: String,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
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
                Err(e) => eprintln!("Failed to parse line: {}. Error: {}", line, e),
            }
        }

        Ok(())
    }

    fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.eq_ignore_ascii_case(level))
            .collect()
    }

    fn filter_by_service(&self, service: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.service.eq_ignore_ascii_case(service))
            .collect()
    }

    fn get_level_summary(&self) -> HashMap<String, usize> {
        let mut summary = HashMap::new();
        for entry in &self.entries {
            *summary.entry(entry.level.clone()).or_insert(0) += 1;
        }
        summary
    }

    fn get_service_summary(&self) -> HashMap<String, usize> {
        let mut summary = HashMap::new();
        for entry in &self.entries {
            *summary.entry(entry.service.clone()).or_insert(0) += 1;
        }
        summary
    }

    fn search_messages(&self, keyword: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.message.to_lowercase().contains(&keyword.to_lowercase()))
            .collect()
    }

    fn export_filtered<P: AsRef<Path>>(
        &self,
        entries: Vec<&LogEntry>,
        path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::create(path)?;
        let entries_vec: Vec<&LogEntry> = entries.into_iter().collect();
        serde_json::to_writer_pretty(file, &entries_vec)?;
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = LogParser::new();
    parser.load_from_file("logs.jsonl")?;

    println!("Total log entries: {}", parser.entries.len());

    let error_logs = parser.filter_by_level("ERROR");
    println!("Error logs count: {}", error_logs.len());

    let api_service_logs = parser.filter_by_service("api-service");
    println!("API service logs count: {}", api_service_logs.len());

    let level_summary = parser.get_level_summary();
    println!("Level summary: {:?}", level_summary);

    let service_summary = parser.get_service_summary();
    println!("Service summary: {:?}", service_summary);

    let search_results = parser.search_messages("timeout");
    println!("Logs containing 'timeout': {}", search_results.len());

    parser.export_filtered(error_logs, "error_logs.json")?;
    println!("Exported error logs to error_logs.json");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_parser() {
        let mut parser = LogParser::new();
        
        let log_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","service":"api-service","message":"Server started successfully"}
{"timestamp":"2024-01-15T10:35:00Z","level":"ERROR","service":"auth-service","message":"Authentication failed"}
{"timestamp":"2024-01-15T10:40:00Z","level":"WARN","service":"api-service","message":"High latency detected"}"#;

        let temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), log_data).unwrap();
        
        parser.load_from_file(temp_file.path()).unwrap();
        assert_eq!(parser.entries.len(), 3);
        
        let error_logs = parser.filter_by_level("ERROR");
        assert_eq!(error_logs.len(), 1);
        
        let api_logs = parser.filter_by_service("api-service");
        assert_eq!(api_logs.len(), 2);
        
        let summary = parser.get_level_summary();
        assert_eq!(summary.get("INFO"), Some(&1));
        assert_eq!(summary.get("ERROR"), Some(&1));
        assert_eq!(summary.get("WARN"), Some(&1));
    }
}