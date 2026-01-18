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
    level_counts: HashMap<String, usize>,
    error_messages: Vec<String>,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
            level_counts: HashMap::new(),
            error_messages: Vec::new(),
        }
    }

    pub fn parse_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            self.parse_line(&line);
        }

        Ok(())
    }

    fn parse_line(&mut self, line: &str) {
        let parts: Vec<&str> = line.splitn(3, ' ').collect();
        if parts.len() == 3 {
            let entry = LogEntry {
                timestamp: parts[0].to_string(),
                level: parts[1].to_string(),
                message: parts[2].to_string(),
            };

            *self.level_counts.entry(entry.level.clone()).or_insert(0) += 1;

            if entry.level == "ERROR" {
                self.error_messages.push(entry.message.clone());
            }

            self.entries.push(entry);
        }
    }

    pub fn get_summary(&self) -> String {
        let total_entries = self.entries.len();
        let mut summary = format!("Total log entries: {}\n", total_entries);
        
        summary.push_str("Level distribution:\n");
        for (level, count) in &self.level_counts {
            let percentage = (*count as f64 / total_entries as f64) * 100.0;
            summary.push_str(&format!("  {}: {} ({:.1}%)\n", level, count, percentage));
        }

        if !self.error_messages.is_empty() {
            summary.push_str("\nError messages:\n");
            for (i, msg) in self.error_messages.iter().enumerate() {
                summary.push_str(&format!("  {}. {}\n", i + 1, msg));
            }
        }

        summary
    }

    pub fn count_by_level(&self, level: &str) -> usize {
        *self.level_counts.get(level).unwrap_or(&0)
    }

    pub fn get_errors(&self) -> &Vec<String> {
        &self.error_messages
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_analysis() {
        let mut log_data = NamedTempFile::new().unwrap();
        writeln!(log_data, "2023-10-01T10:00:00 INFO Application started").unwrap();
        writeln!(log_data, "2023-10-01T10:01:00 WARN High memory usage").unwrap();
        writeln!(log_data, "2023-10-01T10:02:00 ERROR Database connection failed").unwrap();
        writeln!(log_data, "2023-10-01T10:03:00 INFO Request processed").unwrap();

        let mut analyzer = LogAnalyzer::new();
        analyzer.parse_file(log_data.path()).unwrap();

        assert_eq!(analyzer.count_by_level("INFO"), 2);
        assert_eq!(analyzer.count_by_level("WARN"), 1);
        assert_eq!(analyzer.count_by_level("ERROR"), 1);
        assert_eq!(analyzer.get_errors().len(), 1);
    }
}