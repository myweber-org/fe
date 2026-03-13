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
        let mut counts = HashMap::new();
        
        for line in reader.lines() {
            let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
            
            if self.error_pattern.is_match(&line) {
                *counts.entry("ERROR".to_string()).or_insert(0) += 1;
            } else if self.warning_pattern.is_match(&line) {
                *counts.entry("WARN".to_string()).or_insert(0) += 1;
            } else if self.info_pattern.is_match(&line) {
                *counts.entry("INFO".to_string()).or_insert(0) += 1;
            }
        }
        
        Ok(counts)
    }

    pub fn generate_report(&self, counts: &HashMap<String, usize>) -> String {
        let mut report = String::from("Log Analysis Report\n");
        report.push_str("===================\n");
        
        let total: usize = counts.values().sum();
        report.push_str(&format!("Total log entries analyzed: {}\n", total));
        
        for (level, count) in counts {
            let percentage = if total > 0 {
                (*count as f64 / total as f64) * 100.0
            } else {
                0.0
            };
            report.push_str(&format!("{}: {} ({:.1}%)\n", level, count, percentage));
        }
        
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
        writeln!(temp_file, "WARN: Disk space low").unwrap();
        writeln!(temp_file, "ERROR: Database connection failed").unwrap();
        writeln!(temp_file, "INFO: User login successful").unwrap();
        
        let counts = analyzer.analyze_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(counts.get("INFO"), Some(&2));
        assert_eq!(counts.get("WARN"), Some(&1));
        assert_eq!(counts.get("ERROR"), Some(&1));
        
        let report = analyzer.generate_report(&counts);
        assert!(report.contains("Total log entries analyzed: 4"));
        assert!(report.contains("INFO: 2"));
    }
}use std::collections::HashMap;
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
            self.process_line(&line, &mut stats);
        }
        
        Ok(stats)
    }
    
    fn process_line(&self, line: &str, stats: &mut HashMap<String, usize>) {
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
        
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_log_analyzer() {
        let analyzer = LogAnalyzer::new();
        let test_log = "INFO: Application started\nERROR: Database connection failed\nWARN: High memory usage\n";
        
        let mut stats = HashMap::new();
        for line in test_log.lines() {
            analyzer.process_line(line, &mut stats);
        }
        
        assert_eq!(stats.get("info"), Some(&1));
        assert_eq!(stats.get("errors"), Some(&1));
        assert_eq!(stats.get("warnings"), Some(&1));
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
}