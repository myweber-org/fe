use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

#[derive(Debug)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    metadata: HashMap<String, String>,
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

    pub fn parse_file(&mut self, filepath: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);
        let log_pattern = Regex::new(r"\[(?P<timestamp>[^\]]+)\] (?P<level>\w+): (?P<message>.+)")?;

        for line in reader.lines() {
            let line = line?;
            if let Some(captures) = log_pattern.captures(&line) {
                let timestamp = captures.name("timestamp").unwrap().as_str().to_string();
                let level = captures.name("level").unwrap().as_str().to_string();
                let message = captures.name("message").unwrap().as_str().to_string();

                match level.as_str() {
                    "ERROR" => self.error_count += 1,
                    "WARNING" => self.warning_count += 1,
                    _ => {}
                }

                let entry = LogEntry {
                    timestamp,
                    level,
                    message,
                    metadata: HashMap::new(),
                };
                self.entries.push(entry);
            }
        }
        Ok(())
    }

    pub fn generate_report(&self) -> String {
        let total_entries = self.entries.len();
        let mut report = String::new();
        report.push_str(&format!("Log Analysis Report\n"));
        report.push_str(&format!("===================\n"));
        report.push_str(&format!("Total entries: {}\n", total_entries));
        report.push_str(&format!("Error count: {}\n", self.error_count));
        report.push_str(&format!("Warning count: {}\n", self.warning_count));
        
        if total_entries > 0 {
            let error_rate = (self.error_count as f64 / total_entries as f64) * 100.0;
            report.push_str(&format!("Error rate: {:.2}%\n", error_rate));
        }
        
        report
    }

    pub fn get_errors(&self) -> Vec<&LogEntry> {
        self.entries.iter()
            .filter(|entry| entry.level == "ERROR")
            .collect()
    }

    pub fn get_warnings(&self) -> Vec<&LogEntry> {
        self.entries.iter()
            .filter(|entry| entry.level == "WARNING")
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
        let mut log_data = NamedTempFile::new().unwrap();
        writeln!(log_data, "[2023-10-05 14:30:00] INFO: Application started").unwrap();
        writeln!(log_data, "[2023-10-05 14:31:00] ERROR: Database connection failed").unwrap();
        writeln!(log_data, "[2023-10-05 14:32:00] WARNING: High memory usage detected").unwrap();

        let mut analyzer = LogAnalyzer::new();
        analyzer.parse_file(log_data.path().to_str().unwrap()).unwrap();

        assert_eq!(analyzer.entries.len(), 3);
        assert_eq!(analyzer.error_count, 1);
        assert_eq!(analyzer.warning_count, 1);
        
        let errors = analyzer.get_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "Database connection failed");
    }
}