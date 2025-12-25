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

    pub fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let log_pattern = Regex::new(r"(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}) \[(\w+)\] (.+)")?;

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

    pub fn get_statistics(&self) -> &HashMap<String, usize> {
        &self.level_counts
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
    fn test_log_analysis() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "2024-01-15 10:30:00 [INFO] Application started").unwrap();
        writeln!(temp_file, "2024-01-15 10:31:00 [ERROR] Database connection failed").unwrap();
        writeln!(temp_file, "2024-01-15 10:32:00 [WARN] High memory usage detected").unwrap();

        let mut analyzer = LogAnalyzer::new();
        analyzer.load_from_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(analyzer.entries.len(), 3);
        assert_eq!(analyzer.level_counts.get("INFO"), Some(&1));
        assert_eq!(analyzer.filter_by_level("ERROR").len(), 1);
        assert_eq!(analyzer.search_messages("memory").len(), 1);
    }
}