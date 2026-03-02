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
            warning_pattern: Regex::new(r"WARNING").unwrap(),
            info_pattern: Regex::new(r"INFO").unwrap(),
        }
    }

    pub fn analyze_log_file(&self, file_path: &str) -> Result<HashMap<String, usize>, String> {
        let file = File::open(file_path)
            .map_err(|e| format!("Failed to open log file: {}", e))?;
        
        let reader = BufReader::new(file);
        let mut summary = HashMap::new();
        
        for line in reader.lines() {
            let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
            self.process_line(&line, &mut summary);
        }
        
        Ok(summary)
    }

    fn process_line(&self, line: &str, summary: &mut HashMap<String, usize>) {
        if self.error_pattern.is_match(line) {
            *summary.entry("errors".to_string()).or_insert(0) += 1;
        } else if self.warning_pattern.is_match(line) {
            *summary.entry("warnings".to_string()).or_insert(0) += 1;
        } else if self.info_pattern.is_match(line) {
            *summary.entry("info_messages".to_string()).or_insert(0) += 1;
        }
    }

    pub fn generate_report(&self, summary: &HashMap<String, usize>) -> String {
        let mut report = String::from("Log Analysis Report\n");
        report.push_str("===================\n");
        
        for (category, count) in summary {
            report.push_str(&format!("{}: {}\n", category, count));
        }
        
        let total: usize = summary.values().sum();
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
        writeln!(temp_file, "2023-10-01 INFO: Application started").unwrap();
        writeln!(temp_file, "2023-10-01 ERROR: Database connection failed").unwrap();
        writeln!(temp_file, "2023-10-01 WARNING: High memory usage").unwrap();
        writeln!(temp_file, "2023-10-01 INFO: User login successful").unwrap();
        
        let summary = analyzer.analyze_log_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(summary.get("errors"), Some(&1));
        assert_eq!(summary.get("warnings"), Some(&1));
        assert_eq!(summary.get("info_messages"), Some(&2));
    }
}