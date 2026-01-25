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

    pub fn load_from_file(&mut self, path: &str) -> Result<(), std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let log_pattern = Regex::new(r"(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}) \[(\w+)\] (.+)").unwrap();

        for line in reader.lines() {
            let line = line?;
            if let Some(captures) = log_pattern.captures(&line) {
                let entry = LogEntry {
                    timestamp: captures[1].to_string(),
                    level: captures[2].to_string(),
                    message: captures[3].to_string(),
                };
                
                *self.level_counts.entry(entry.level.clone()).or_insert(0) += 1;
                self.entries.push(entry);
            }
        }
        Ok(())
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn get_summary(&self) -> HashMap<String, usize> {
        self.level_counts.clone()
    }

    pub fn search_messages(&self, pattern: &str) -> Vec<&LogEntry> {
        let search_regex = Regex::new(pattern).unwrap_or_else(|_| Regex::new(".*").unwrap());
        self.entries
            .iter()
            .filter(|entry| search_regex.is_match(&entry.message))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_analyzer_creation() {
        let analyzer = LogAnalyzer::new();
        assert_eq!(analyzer.entries.len(), 0);
    }

    #[test]
    fn test_filter_by_level() {
        let mut analyzer = LogAnalyzer::new();
        analyzer.entries.push(LogEntry {
            timestamp: "2024-01-01 10:00:00".to_string(),
            level: "ERROR".to_string(),
            message: "Test error".to_string(),
        });
        
        let errors = analyzer.filter_by_level("ERROR");
        assert_eq!(errors.len(), 1);
    }
}