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
}