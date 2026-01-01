use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
}

pub struct LogAnalyzer {
    entries: Vec<LogEntry>,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if let Some(entry) = Self::parse_log_line(&line) {
                self.entries.push(entry);
            }
        }
        Ok(())
    }

    fn parse_log_line(line: &str) -> Option<LogEntry> {
        let parts: Vec<&str> = line.splitn(3, ' ').collect();
        if parts.len() == 3 {
            Some(LogEntry {
                timestamp: parts[0].to_string(),
                level: parts[1].to_string(),
                message: parts[2].to_string(),
            })
        } else {
            None
        }
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn count_by_level(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for entry in &self.entries {
            *counts.entry(entry.level.clone()).or_insert(0) += 1;
        }
        counts
    }

    pub fn get_summary(&self) -> String {
        let total = self.entries.len();
        let counts = self.count_by_level();
        let mut summary = format!("Total log entries: {}\n", total);
        
        for (level, count) in counts {
            summary.push_str(&format!("{}: {}\n", level, count));
        }
        
        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_log_line() {
        let line = "2023-10-01T12:00:00 INFO User logged in";
        let entry = LogAnalyzer::parse_log_line(line).unwrap();
        
        assert_eq!(entry.timestamp, "2023-10-01T12:00:00");
        assert_eq!(entry.level, "INFO");
        assert_eq!(entry.message, "User logged in");
    }

    #[test]
    fn test_filter_by_level() {
        let mut analyzer = LogAnalyzer::new();
        analyzer.entries.push(LogEntry {
            timestamp: "2023-10-01T12:00:00".to_string(),
            level: "INFO".to_string(),
            message: "Test message".to_string(),
        });
        analyzer.entries.push(LogEntry {
            timestamp: "2023-10-01T12:01:00".to_string(),
            level: "ERROR".to_string(),
            message: "Error occurred".to_string(),
        });

        let info_entries = analyzer.filter_by_level("INFO");
        assert_eq!(info_entries.len(), 1);
        assert_eq!(info_entries[0].level, "INFO");
    }
}