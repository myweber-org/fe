use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    service: String,
    message: String,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug)]
pub struct LogParser {
    entries: Vec<LogEntry>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            entries: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
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

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .collect()
    }

    pub fn filter_by_service(&self, service: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.service == service)
            .collect()
    }

    pub fn get_summary(&self) -> HashMap<String, usize> {
        let mut summary = HashMap::new();
        
        for entry in &self.entries {
            *summary.entry(entry.level.clone()).or_insert(0) += 1;
        }
        
        summary
    }

    pub fn get_service_summary(&self) -> HashMap<String, usize> {
        let mut summary = HashMap::new();
        
        for entry in &self.entries {
            *summary.entry(entry.service.clone()).or_insert(0) += 1;
        }
        
        summary
    }

    pub fn search_messages(&self, keyword: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.message.to_lowercase().contains(&keyword.to_lowercase()))
            .collect()
    }

    pub fn count_entries(&self) -> usize {
        self.entries.len()
    }

    pub fn get_entries(&self) -> &[LogEntry] {
        &self.entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_log_file() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        
        let log_lines = vec![
            r#"{"timestamp":"2024-01-15T10:30:00Z","level":"ERROR","service":"auth","message":"Authentication failed","user_id":123}"#,
            r#"{"timestamp":"2024-01-15T10:31:00Z","level":"INFO","service":"api","message":"Request processed","duration_ms":45}"#,
            r#"{"timestamp":"2024-01-15T10:32:00Z","level":"WARN","service":"database","message":"Slow query detected","query_time":2.5}"#,
            r#"{"timestamp":"2024-01-15T10:33:00Z","level":"ERROR","service":"auth","message":"Invalid token","ip":"192.168.1.1"}"#,
            r#"{"timestamp":"2024-01-15T10:34:00Z","level":"INFO","service":"api","message":"Cache hit","key":"user:456"}"#,
        ];

        for line in log_lines {
            writeln!(file, "{}", line).unwrap();
        }

        file
    }

    #[test]
    fn test_load_and_count() {
        let file = create_test_log_file();
        let mut parser = LogParser::new();
        
        parser.load_from_file(file.path()).unwrap();
        assert_eq!(parser.count_entries(), 5);
    }

    #[test]
    fn test_filter_by_level() {
        let file = create_test_log_file();
        let mut parser = LogParser::new();
        
        parser.load_from_file(file.path()).unwrap();
        let errors = parser.filter_by_level("ERROR");
        assert_eq!(errors.len(), 2);
        
        let infos = parser.filter_by_level("INFO");
        assert_eq!(infos.len(), 2);
    }

    #[test]
    fn test_filter_by_service() {
        let file = create_test_log_file();
        let mut parser = LogParser::new();
        
        parser.load_from_file(file.path()).unwrap();
        let auth_entries = parser.filter_by_service("auth");
        assert_eq!(auth_entries.len(), 2);
    }

    #[test]
    fn test_summary() {
        let file = create_test_log_file();
        let mut parser = LogParser::new();
        
        parser.load_from_file(file.path()).unwrap();
        let summary = parser.get_summary();
        
        assert_eq!(summary.get("ERROR"), Some(&2));
        assert_eq!(summary.get("INFO"), Some(&2));
        assert_eq!(summary.get("WARN"), Some(&1));
    }

    #[test]
    fn test_search_messages() {
        let file = create_test_log_file();
        let mut parser = LogParser::new();
        
        parser.load_from_file(file.path()).unwrap();
        let failed_entries = parser.search_messages("failed");
        assert_eq!(failed_entries.len(), 1);
        
        let auth_entries = parser.search_messages("auth");
        assert_eq!(auth_entries.len(), 1);
    }
}