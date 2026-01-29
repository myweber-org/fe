use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

pub struct LogAnalyzer {
    error_pattern: Regex,
    warning_pattern: Regex,
    info_pattern: Regex,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            error_pattern: Regex::new(r"ERROR").unwrap(),
            warning_pattern: Regex::new(r"WARN").unwrap(),
            info_pattern: Regex::new(r"INFO").unwrap(),
        }
    }

    pub fn analyze_file(&self, file_path: &str) -> Result<HashMap<String, usize>, String> {
        let file = File::open(file_path)
            .map_err(|e| format!("Failed to open file: {}", e))?;
        
        let reader = BufReader::new(file);
        let mut stats = HashMap::new();
        
        for line in reader.lines() {
            let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
            self.analyze_line(&line, &mut stats);
        }
        
        Ok(stats)
    }

    fn analyze_line(&self, line: &str, stats: &mut HashMap<String, usize>) {
        if self.error_pattern.is_match(line) {
            *stats.entry("errors".to_string()).or_insert(0) += 1;
        } else if self.warning_pattern.is_match(line) {
            *stats.entry("warnings".to_string()).or_insert(0) += 1;
        } else if self.info_pattern.is_match(line) {
            *stats.entry("info".to_string()).or_insert(0) += 1;
        }
    }

    pub fn generate_report(&self, stats: &HashMap<String, usize>) -> String {
        let mut report = String::from("Log Analysis Report\n");
        report.push_str("===================\n");
        
        for (category, count) in stats {
            report.push_str(&format!("{}: {}\n", category, count));
        }
        
        let total: usize = stats.values().sum();
        report.push_str(&format!("\nTotal log entries analyzed: {}", total));
        
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_analysis() {
        let analyzer = LogAnalyzer::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "INFO: Application started").unwrap();
        writeln!(temp_file, "WARN: Deprecated API used").unwrap();
        writeln!(temp_file, "ERROR: Database connection failed").unwrap();
        writeln!(temp_file, "INFO: User logged in").unwrap();
        
        let stats = analyzer.analyze_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(stats.get("info"), Some(&2));
        assert_eq!(stats.get("warnings"), Some(&1));
        assert_eq!(stats.get("errors"), Some(&1));
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

    pub fn parse_file(&mut self, filepath: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(filepath)?;
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
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn total_entries(&self) -> usize {
        self.entries.len()
    }

    pub fn contains_error(&self) -> bool {
        self.level_counts.contains_key("ERROR")
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
        writeln!(
            log_data,
            "2023-10-05 14:30:00 [INFO] Application started\n\
             2023-10-05 14:31:00 [WARN] High memory usage detected\n\
             2023-10-05 14:32:00 [ERROR] Database connection failed"
        )
        .unwrap();

        let mut analyzer = LogAnalyzer::new();
        analyzer
            .parse_file(log_data.path().to_str().unwrap())
            .unwrap();

        assert_eq!(analyzer.total_entries(), 3);
        assert!(analyzer.contains_error());
        assert_eq!(analyzer.filter_by_level("INFO").len(), 1);
    }
}