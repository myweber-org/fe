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
        } else {
            *stats.entry("other".to_string()).or_insert(0) += 1;
        }
    }

    pub fn generate_report(&self, stats: &HashMap<String, usize>) -> String {
        let total: usize = stats.values().sum();
        let mut report = format!("Log Analysis Report\n");
        report.push_str(&format!("Total entries: {}\n", total));
        
        for (category, count) in stats {
            let percentage = (*count as f64 / total as f64) * 100.0;
            report.push_str(&format!("{}: {} ({:.1}%)\n", category, count, percentage));
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
        writeln!(temp_file, "WARNING: Disk space low").unwrap();
        writeln!(temp_file, "ERROR: Connection failed").unwrap();
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

pub struct LogAnalyzer {
    error_patterns: HashMap<String, usize>,
    warning_patterns: HashMap<String, usize>,
    total_lines: usize,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            error_patterns: HashMap::new(),
            warning_patterns: HashMap::new(),
            total_lines: 0,
        }
    }

    pub fn analyze_file(&mut self, file_path: &str) -> Result<(), std::io::Error> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let error_regex = Regex::new(r"ERROR: (.+)").unwrap();
        let warning_regex = Regex::new(r"WARNING: (.+)").unwrap();

        for line in reader.lines() {
            let line = line?;
            self.total_lines += 1;

            if let Some(caps) = error_regex.captures(&line) {
                let error_msg = caps.get(1).unwrap().as_str().to_string();
                *self.error_patterns.entry(error_msg).or_insert(0) += 1;
            } else if let Some(caps) = warning_regex.captures(&line) {
                let warning_msg = caps.get(1).unwrap().as_str().to_string();
                *self.warning_patterns.entry(warning_msg).or_insert(0) += 1;
            }
        }

        Ok(())
    }

    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str(&format!("Total log lines analyzed: {}\n", self.total_lines));
        report.push_str("\nError Summary:\n");
        
        if self.error_patterns.is_empty() {
            report.push_str("No errors found\n");
        } else {
            for (error, count) in &self.error_patterns {
                report.push_str(&format!("  {}: {} occurrences\n", error, count));
            }
        }

        report.push_str("\nWarning Summary:\n");
        if self.warning_patterns.is_empty() {
            report.push_str("No warnings found\n");
        } else {
            for (warning, count) in &self.warning_patterns {
                report.push_str(&format!("  {}: {} occurrences\n", warning, count));
            }
        }

        report
    }

    pub fn get_top_errors(&self, limit: usize) -> Vec<(String, usize)> {
        let mut errors: Vec<_> = self.error_patterns.iter().collect();
        errors.sort_by(|a, b| b.1.cmp(a.1));
        errors.iter()
            .take(limit)
            .map(|(msg, count)| (msg.to_string(), *count))
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
        let mut log_data = String::new();
        log_data.push_str("INFO: Application started\n");
        log_data.push_str("WARNING: Disk space running low\n");
        log_data.push_str("ERROR: Database connection failed\n");
        log_data.push_str("WARNING: Disk space running low\n");
        log_data.push_str("ERROR: Database connection failed\n");
        log_data.push_str("ERROR: Invalid user input\n");

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", log_data).unwrap();

        let mut analyzer = LogAnalyzer::new();
        analyzer.analyze_file(temp_file.path().to_str().unwrap()).unwrap();

        let report = analyzer.generate_report();
        assert!(report.contains("Total log lines analyzed: 6"));
        assert!(report.contains("Database connection failed: 2 occurrences"));
        assert!(report.contains("Disk space running low: 2 occurrences"));

        let top_errors = analyzer.get_top_errors(2);
        assert_eq!(top_errors.len(), 2);
        assert_eq!(top_errors[0].0, "Database connection failed");
        assert_eq!(top_errors[0].1, 2);
    }
}