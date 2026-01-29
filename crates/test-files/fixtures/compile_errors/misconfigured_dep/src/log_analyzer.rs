use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use regex::Regex;

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

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let log_pattern = Regex::new(r"\[(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2})\] (\w+): (.+)").unwrap();

        for line in reader.lines() {
            let line = line?;
            if let Some(captures) = log_pattern.captures(&line) {
                let entry = LogEntry {
                    timestamp: captures[1].to_string(),
                    level: captures[2].to_string(),
                    message: captures[3].to_string(),
                };
                self.entries.push(entry);
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

    pub fn count_entries(&self) -> usize {
        self.entries.len()
    }

    pub fn get_errors(&self) -> Vec<&LogEntry> {
        self.filter_by_level("ERROR")
    }

    pub fn get_warnings(&self) -> Vec<&LogEntry> {
        self.filter_by_level("WARN")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_parsing() {
        let mut log_data = NamedTempFile::new().unwrap();
        writeln!(log_data, "[2024-01-15 10:30:45] INFO: Application started").unwrap();
        writeln!(log_data, "[2024-01-15 10:31:22] ERROR: Database connection failed").unwrap();
        writeln!(log_data, "[2024-01-15 10:32:10] WARN: High memory usage detected").unwrap();

        let mut analyzer = LogAnalyzer::new();
        analyzer.load_from_file(log_data.path()).unwrap();

        assert_eq!(analyzer.count_entries(), 3);
        assert_eq!(analyzer.get_errors().len(), 1);
        assert_eq!(analyzer.get_warnings().len(), 1);
    }
}