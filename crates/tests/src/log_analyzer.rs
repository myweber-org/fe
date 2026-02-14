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
        
        summary.insert("total_lines".to_string(), 0);
        summary.insert("errors".to_string(), 0);
        summary.insert("warnings".to_string(), 0);
        summary.insert("info_messages".to_string(), 0);

        for line in reader.lines() {
            let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
            
            *summary.get_mut("total_lines").unwrap() += 1;
            
            if self.error_pattern.is_match(&line) {
                *summary.get_mut("errors").unwrap() += 1;
            } else if self.warning_pattern.is_match(&line) {
                *summary.get_mut("warnings").unwrap() += 1;
            } else if self.info_pattern.is_match(&line) {
                *summary.get_mut("info_messages").unwrap() += 1;
            }
        }
        
        Ok(summary)
    }
    
    pub fn generate_report(&self, summary: &HashMap<String, usize>) -> String {
        let mut report = String::new();
        report.push_str("Log Analysis Report\n");
        report.push_str("===================\n");
        
        for (key, value) in summary {
            report.push_str(&format!("{}: {}\n", key, value));
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
        writeln!(temp_file, "WARNING: Low memory").unwrap();
        writeln!(temp_file, "ERROR: Connection failed").unwrap();
        writeln!(temp_file, "INFO: Processing data").unwrap();
        
        let summary = analyzer.analyze_log_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(summary.get("total_lines"), Some(&4));
        assert_eq!(summary.get("errors"), Some(&1));
        assert_eq!(summary.get("warnings"), Some(&1));
        assert_eq!(summary.get("info_messages"), Some(&2));
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
            *stats.entry("ERROR".to_string()).or_insert(0) += 1;
        } else if self.warn_pattern.is_match(line) {
            *stats.entry("WARN".to_string()).or_insert(0) += 1;
        } else if self.info_pattern.is_match(line) {
            *stats.entry("INFO".to_string()).or_insert(0) += 1;
        }
    }
    
    pub fn print_summary(&self, stats: &HashMap<String, usize>) {
        println!("Log Analysis Summary:");
        println!("=====================");
        
        let total: usize = stats.values().sum();
        
        for (level, count) in stats {
            let percentage = (*count as f64 / total as f64) * 100.0;
            println!("{}: {} ({:.1}%)", level, count, percentage);
        }
        
        println!("Total entries: {}", total);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_log_analyzer() {
        let analyzer = LogAnalyzer::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "2024-01-15 INFO: Application started").unwrap();
        writeln!(temp_file, "2024-01-15 WARN: Low disk space").unwrap();
        writeln!(temp_file, "2024-01-15 ERROR: Database connection failed").unwrap();
        writeln!(temp_file, "2024-01-15 INFO: User logged in").unwrap();
        
        let stats = analyzer.analyze_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(stats.get("INFO"), Some(&2));
        assert_eq!(stats.get("WARN"), Some(&1));
        assert_eq!(stats.get("ERROR"), Some(&1));
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
            *stats.entry("ERROR".to_string()).or_insert(0) += 1;
        } else if self.warn_pattern.is_match(line) {
            *stats.entry("WARN".to_string()).or_insert(0) += 1;
        } else if self.info_pattern.is_match(line) {
            *stats.entry("INFO".to_string()).or_insert(0) += 1;
        }
    }
    
    pub fn generate_report(&self, stats: &HashMap<String, usize>) -> String {
        let mut report = String::from("Log Analysis Report\n");
        report.push_str("===================\n");
        
        for (level, count) in stats {
            report.push_str(&format!("{}: {}\n", level, count));
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
        writeln!(temp_file, "2023-01-01 INFO: Application started").unwrap();
        writeln!(temp_file, "2023-01-01 WARN: Low memory").unwrap();
        writeln!(temp_file, "2023-01-01 ERROR: Database connection failed").unwrap();
        writeln!(temp_file, "2023-01-01 INFO: User logged in").unwrap();
        
        let stats = analyzer.analyze_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(stats.get("INFO"), Some(&2));
        assert_eq!(stats.get("WARN"), Some(&1));
        assert_eq!(stats.get("ERROR"), Some(&1));
    }
}