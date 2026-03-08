use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

#[derive(Debug, PartialEq, Eq, Hash)]
enum LogLevel {
    Error,
    Warning,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "ERROR" => Some(LogLevel::Error),
            "WARNING" => Some(LogLevel::Warning),
            "INFO" => Some(LogLevel::Info),
            "DEBUG" => Some(LogLevel::Debug),
            "TRACE" => Some(LogLevel::Trace),
            _ => None,
        }
    }
}

pub struct LogAnalyzer {
    counts: HashMap<LogLevel, u32>,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            counts: HashMap::new(),
        }
    }

    pub fn analyze_file<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let file = File::open(path)?;
        let reader = io::BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            self.process_line(&line);
        }

        Ok(())
    }

    fn process_line(&mut self, line: &str) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            if let Some(level) = LogLevel::from_str(parts[0]) {
                *self.counts.entry(level).or_insert(0) += 1;
            }
        }
    }

    pub fn get_counts(&self) -> &HashMap<LogLevel, u32> {
        &self.counts
    }

    pub fn total_errors(&self) -> u32 {
        self.counts.get(&LogLevel::Error).copied().unwrap_or(0)
    }

    pub fn summary(&self) -> String {
        let mut result = String::from("Log Analysis Summary:\n");
        let levels = [
            LogLevel::Error,
            LogLevel::Warning,
            LogLevel::Info,
            LogLevel::Debug,
            LogLevel::Trace,
        ];

        for level in levels.iter() {
            if let Some(count) = self.counts.get(level) {
                result.push_str(&format!("{:?}: {}\n", level, count));
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_analyzer() {
        let mut analyzer = LogAnalyzer::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "ERROR Database connection failed").unwrap();
        writeln!(temp_file, "WARNING High memory usage").unwrap();
        writeln!(temp_file, "INFO User logged in").unwrap();
        writeln!(temp_file, "ERROR File not found").unwrap();
        writeln!(temp_file, "INFO Request processed").unwrap();

        analyzer.analyze_file(temp_file.path()).unwrap();
        
        let counts = analyzer.get_counts();
        assert_eq!(counts.get(&LogLevel::Error), Some(&2));
        assert_eq!(counts.get(&LogLevel::Warning), Some(&1));
        assert_eq!(counts.get(&LogLevel::Info), Some(&2));
        assert_eq!(analyzer.total_errors(), 2);
    }

    #[test]
    fn test_invalid_log_level() {
        let mut analyzer = LogAnalyzer::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "UNKNOWN Some message").unwrap();
        writeln!(temp_file, "INFO Valid message").unwrap();

        analyzer.analyze_file(temp_file.path()).unwrap();
        
        let counts = analyzer.get_counts();
        assert_eq!(counts.get(&LogLevel::Info), Some(&1));
        assert_eq!(counts.len(), 1);
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
        
        let stats = analyzer.analyze_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(stats.get("info"), Some(&2));
        assert_eq!(stats.get("warnings"), Some(&1));
        assert_eq!(stats.get("errors"), Some(&1));
    }
}