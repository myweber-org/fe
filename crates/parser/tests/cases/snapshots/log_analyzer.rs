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
        writeln!(
            temp_file,
            "2024-01-15 10:30:00 [INFO] Application started"
        ).unwrap();
        writeln!(
            temp_file,
            "2024-01-15 10:31:00 [ERROR] Failed to connect to database"
        ).unwrap();
        writeln!(
            temp_file,
            "2024-01-15 10:32:00 [WARN] High memory usage detected"
        ).unwrap();

        let mut analyzer = LogAnalyzer::new();
        analyzer.parse_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(analyzer.total_entries(), 3);
        assert_eq!(analyzer.get_level_summary().get("INFO"), Some(&1));
        assert_eq!(analyzer.get_level_summary().get("ERROR"), Some(&1));
        assert_eq!(analyzer.get_level_summary().get("WARN"), Some(&1));
    }
}use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

pub struct LogAnalyzer {
    error_pattern: Regex,
    warn_pattern: Regex,
    info_pattern: Regex,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            error_pattern: Regex::new(r"ERROR").unwrap(),
            warn_pattern: Regex::new(r"WARN").unwrap(),
            info_pattern: Regex::new(r"INFO").unwrap(),
        }
    }

    pub fn analyze_file(&self, path: &str) -> Result<HashMap<String, usize>, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);
        
        let mut stats = HashMap::new();
        stats.insert("total_lines".to_string(), 0);
        stats.insert("errors".to_string(), 0);
        stats.insert("warnings".to_string(), 0);
        stats.insert("info".to_string(), 0);

        for line_result in reader.lines() {
            let line = line_result.map_err(|e| format!("Failed to read line: {}", e))?;
            
            *stats.get_mut("total_lines").unwrap() += 1;
            
            if self.error_pattern.is_match(&line) {
                *stats.get_mut("errors").unwrap() += 1;
            } else if self.warn_pattern.is_match(&line) {
                *stats.get_mut("warnings").unwrap() += 1;
            } else if self.info_pattern.is_match(&line) {
                *stats.get_mut("info").unwrap() += 1;
            }
        }

        Ok(stats)
    }

    pub fn generate_report(&self, stats: &HashMap<String, usize>) -> String {
        let total = stats.get("total_lines").unwrap_or(&0);
        let errors = stats.get("errors").unwrap_or(&0);
        let warnings = stats.get("warnings").unwrap_or(&0);
        let info = stats.get("info").unwrap_or(&0);

        format!(
            "Log Analysis Report:\n\
             Total Lines: {}\n\
             Errors: {}\n\
             Warnings: {}\n\
             Info Messages: {}\n\
             Error Rate: {:.2}%",
            total,
            errors,
            warnings,
            info,
            (*errors as f64 / *total as f64) * 100.0
        )
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
        writeln!(temp_file, "WARN: Low memory").unwrap();
        writeln!(temp_file, "ERROR: Connection failed").unwrap();
        writeln!(temp_file, "INFO: Processing data").unwrap();
        
        let stats = analyzer.analyze_file(temp_file.path().to_str().unwrap()).unwrap();
        let report = analyzer.generate_report(&stats);
        
        assert_eq!(*stats.get("total_lines").unwrap(), 4);
        assert_eq!(*stats.get("errors").unwrap(), 1);
        assert_eq!(*stats.get("warnings").unwrap(), 1);
        assert_eq!(*stats.get("info").unwrap(), 2);
        assert!(report.contains("Error Rate: 25.00%"));
    }
}