use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use regex::Regex;

#[derive(Debug)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    source: String,
}

pub struct LogAnalyzer {
    entries: Vec<LogEntry>,
    level_counts: HashMap<String, usize>,
    source_counts: HashMap<String, usize>,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
            level_counts: HashMap::new(),
            source_counts: HashMap::new(),
        }
    }

    pub fn load_from_file(&mut self, path: &str) -> io::Result<()> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let log_pattern = Regex::new(r"\[(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2})\] (\w+) \[(\w+)\]: (.+)").unwrap();

        for line in reader.lines() {
            let line = line?;
            if let Some(caps) = log_pattern.captures(&line) {
                let entry = LogEntry {
                    timestamp: caps[1].to_string(),
                    level: caps[2].to_string(),
                    source: caps[3].to_string(),
                    message: caps[4].to_string(),
                };

                *self.level_counts.entry(entry.level.clone()).or_insert(0) += 1;
                *self.source_counts.entry(entry.source.clone()).or_insert(0) += 1;
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

    pub fn filter_by_source(&self, source: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.source == source)
            .collect()
    }

    pub fn get_summary(&self) -> String {
        let total_entries = self.entries.len();
        let error_count = self.level_counts.get("ERROR").unwrap_or(&0);
        let warning_count = self.level_counts.get("WARNING").unwrap_or(&0);
        let info_count = self.level_counts.get("INFO").unwrap_or(&0);

        format!(
            "Total entries: {}\nErrors: {}\nWarnings: {}\nInfo: {}",
            total_entries, error_count, warning_count, info_count
        )
    }

    pub fn get_top_sources(&self, n: usize) -> Vec<(&String, &usize)> {
        let mut sources: Vec<_> = self.source_counts.iter().collect();
        sources.sort_by(|a, b| b.1.cmp(a.1));
        sources.into_iter().take(n).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_analyzer() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "[2024-01-15 10:30:45] ERROR [AuthService]: Authentication failed").unwrap();
        writeln!(temp_file, "[2024-01-15 10:31:22] INFO [Database]: Connection established").unwrap();
        writeln!(temp_file, "[2024-01-15 10:32:10] WARNING [Cache]: Memory usage high").unwrap();

        let mut analyzer = LogAnalyzer::new();
        analyzer.load_from_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(analyzer.entries.len(), 3);
        assert_eq!(analyzer.filter_by_level("ERROR").len(), 1);
        assert_eq!(analyzer.filter_by_source("Database").len(), 1);
        
        let summary = analyzer.get_summary();
        assert!(summary.contains("Total entries: 3"));
    }
}use std::collections::HashMap;
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

    pub fn parse_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let log_pattern = Regex::new(r"\[(.*?)\] (\w+): (.*)")?;

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
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn total_entries(&self) -> usize {
        self.entries.len()
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
        writeln!(temp_file, "[2023-10-01 10:00:00] INFO: Application started").unwrap();
        writeln!(temp_file, "[2023-10-01 10:00:05] ERROR: Database connection failed").unwrap();
        writeln!(temp_file, "[2023-10-01 10:00:10] WARN: High memory usage detected").unwrap();

        let mut analyzer = LogAnalyzer::new();
        analyzer.parse_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(analyzer.total_entries(), 3);
        assert_eq!(analyzer.get_level_summary().get("INFO"), Some(&1));
        assert_eq!(analyzer.get_level_summary().get("ERROR"), Some(&1));
        assert_eq!(analyzer.get_level_summary().get("WARN"), Some(&1));
    }
}