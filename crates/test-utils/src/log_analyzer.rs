use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

#[derive(Debug)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
}

pub struct LogAnalyzer {
    entries: Vec<LogEntry>,
    level_counts: HashMap<String, usize>,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
            level_counts: HashMap::new(),
        }
    }

    pub fn parse_file(&mut self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let log_pattern = Regex::new(r"(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}) \[(\w+)\] (.+)")?;

        for line in reader.lines() {
            let line = line?;
            if let Some(captures) = log_pattern.captures(&line) {
                let timestamp = captures[1].to_string();
                let level = captures[2].to_string();
                let message = captures[3].to_string();

                let entry = LogEntry {
                    timestamp,
                    level: level.clone(),
                    message,
                };

                self.entries.push(entry);
                *self.level_counts.entry(level).or_insert(0) += 1;
            }
        }

        Ok(())
    }

    pub fn get_level_summary(&self) -> &HashMap<String, usize> {
        &self.level_counts
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .collect()
    }

    pub fn total_entries(&self) -> usize {
        self.entries.len()
    }

    pub fn search_messages(&self, keyword: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.message.contains(keyword))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "2024-01-15 10:30:00 [INFO] Application started").unwrap();
        writeln!(temp_file, "2024-01-15 10:31:00 [ERROR] Failed to connect to database").unwrap();
        writeln!(temp_file, "2024-01-15 10:32:00 [WARN] High memory usage detected").unwrap();

        let mut analyzer = LogAnalyzer::new();
        analyzer.parse_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(analyzer.total_entries(), 3);
        assert_eq!(analyzer.get_level_summary().get("INFO"), Some(&1));
        assert_eq!(analyzer.get_level_summary().get("ERROR"), Some(&1));
        assert_eq!(analyzer.get_level_summary().get("WARN"), Some(&1));
    }

    #[test]
    fn test_filter_and_search() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "2024-01-15 10:30:00 [INFO] User login successful").unwrap();
        writeln!(temp_file, "2024-01-15 10:31:00 [ERROR] Database connection failed").unwrap();
        writeln!(temp_file, "2024-01-15 10:32:00 [INFO] User logout").unwrap();

        let mut analyzer = LogAnalyzer::new();
        analyzer.parse_file(temp_file.path().to_str().unwrap()).unwrap();

        let info_entries = analyzer.filter_by_level("INFO");
        assert_eq!(info_entries.len(), 2);

        let user_entries = analyzer.search_messages("User");
        assert_eq!(user_entries.len(), 2);

        let db_entries = analyzer.search_messages("Database");
        assert_eq!(db_entries.len(), 1);
    }
}use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use chrono::{DateTime, FixedOffset};

#[derive(Debug)]
pub struct LogEntry {
    timestamp: DateTime<FixedOffset>,
    level: String,
    component: String,
    message: String,
    metadata: HashMap<String, String>,
}

impl LogEntry {
    pub fn new(timestamp: DateTime<FixedOffset>, level: &str, component: &str, message: &str) -> Self {
        LogEntry {
            timestamp,
            level: level.to_string(),
            component: component.to_string(),
            message: message.to_string(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }

    pub fn is_error(&self) -> bool {
        self.level == "ERROR" || self.level == "FATAL"
    }
}

pub struct LogAnalyzer {
    entries: Vec<LogEntry>,
    error_count: usize,
    warning_count: usize,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
            error_count: 0,
            warning_count: 0,
        }
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            self.parse_line(&line)?;
        }

        Ok(())
    }

    fn parse_line(&mut self, line: &str) -> Result<(), Box<dyn std::error::Error>> {
        let parts: Vec<&str> = line.splitn(4, '|').collect();
        if parts.len() < 4 {
            return Ok(());
        }

        let timestamp = DateTime::parse_from_rfc3339(parts[0])?;
        let level = parts[1];
        let component = parts[2];
        let message = parts[3];

        let mut entry = LogEntry::new(timestamp, level, component, message);

        if entry.is_error() {
            self.error_count += 1;
        } else if level == "WARN" {
            self.warning_count += 1;
        }

        self.entries.push(entry);
        Ok(())
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn filter_by_component(&self, component: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.component == component)
            .collect()
    }

    pub fn get_statistics(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        stats.insert("total".to_string(), self.entries.len());
        stats.insert("errors".to_string(), self.error_count);
        stats.insert("warnings".to_string(), self.warning_count);

        let mut component_counts = HashMap::new();
        for entry in &self.entries {
            *component_counts.entry(entry.component.clone()).or_insert(0) += 1;
        }

        for (component, count) in component_counts {
            stats.insert(format!("component_{}", component), count);
        }

        stats
    }

    pub fn find_pattern(&self, pattern: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.message.contains(pattern))
            .collect()
    }
}

pub fn analyze_log_file(path: &str) -> Result<HashMap<String, usize>, Box<dyn std::error::Error>> {
    let mut analyzer = LogAnalyzer::new();
    analyzer.load_from_file(path)?;
    Ok(analyzer.get_statistics())
}