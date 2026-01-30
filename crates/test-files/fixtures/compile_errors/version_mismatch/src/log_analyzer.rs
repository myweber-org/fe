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

    pub fn analyze_file(&self, path: &str) -> Result<HashMap<String, usize>, std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut stats = HashMap::new();

        for line in reader.lines() {
            let line = line?;
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
            *stats.entry("info_messages".to_string()).or_insert(0) += 1;
        }
    }

    pub fn generate_report(&self, stats: &HashMap<String, usize>) -> String {
        let mut report = String::new();
        report.push_str("Log Analysis Report\n");
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

    #[test]
    fn test_log_analysis() {
        let analyzer = LogAnalyzer::new();
        let test_log = "INFO: Application started\nERROR: Database connection failed\nWARN: High memory usage\nINFO: User login successful";
        
        let mut stats = HashMap::new();
        for line in test_log.lines() {
            analyzer.process_line(line, &mut stats);
        }

        assert_eq!(stats.get("errors"), Some(&1));
        assert_eq!(stats.get("warnings"), Some(&1));
        assert_eq!(stats.get("info_messages"), Some(&2));
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

        for line in reader.lines() {
            let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
            
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
        format!(
            "Log Analysis Report:\n\
            Total Lines: {}\n\
            Errors: {}\n\
            Warnings: {}\n\
            Info Messages: {}",
            stats.get("total_lines").unwrap_or(&0),
            stats.get("errors").unwrap_or(&0),
            stats.get("warnings").unwrap_or(&0),
            stats.get("info").unwrap_or(&0)
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
        
        assert_eq!(*stats.get("total_lines").unwrap(), 4);
        assert_eq!(*stats.get("errors").unwrap(), 1);
        assert_eq!(*stats.get("warnings").unwrap(), 1);
        assert_eq!(*stats.get("info").unwrap(), 2);
    }
}