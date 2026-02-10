use std::collections::HashMap;
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

    pub fn analyze_log_file(&self, file_path: &str) -> Result<HashMap<String, usize>, String> {
        let file = File::open(file_path)
            .map_err(|e| format!("Failed to open log file: {}", e))?;
        
        let reader = BufReader::new(file);
        let mut stats = HashMap::new();
        
        stats.insert("total_lines".to_string(), 0);
        stats.insert("errors".to_string(), 0);
        stats.insert("warnings".to_string(), 0);
        stats.insert("info_messages".to_string(), 0);

        for line_result in reader.lines() {
            let line = line_result.map_err(|e| format!("Failed to read line: {}", e))?;
            
            *stats.get_mut("total_lines").unwrap() += 1;
            
            if self.error_pattern.is_match(&line) {
                *stats.get_mut("errors").unwrap() += 1;
            } else if self.warn_pattern.is_match(&line) {
                *stats.get_mut("warnings").unwrap() += 1;
            } else if self.info_pattern.is_match(&line) {
                *stats.get_mut("info_messages").unwrap() += 1;
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
            stats.get("info_messages").unwrap_or(&0)
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
        writeln!(temp_file, "WARN: Disk space running low").unwrap();
        writeln!(temp_file, "ERROR: Database connection failed").unwrap();
        writeln!(temp_file, "INFO: User login successful").unwrap();
        
        let stats = analyzer.analyze_log_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(stats.get("total_lines"), Some(&4));
        assert_eq!(stats.get("errors"), Some(&1));
        assert_eq!(stats.get("warnings"), Some(&1));
        assert_eq!(stats.get("info_messages"), Some(&2));
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
    source: String,
}

#[derive(Debug)]
pub struct LogSummary {
    total_entries: usize,
    error_count: usize,
    warning_count: usize,
    info_count: usize,
    source_distribution: HashMap<String, usize>,
    recent_errors: Vec<LogEntry>,
}

pub struct LogAnalyzer {
    log_entries: Vec<LogEntry>,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            log_entries: Vec::new(),
        }
    }

    pub fn load_from_file(&mut self, file_path: &str) -> Result<(), std::io::Error> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let log_pattern = Regex::new(r"^(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}) \[(\w+)\] (\w+): (.+)$").unwrap();

        for line in reader.lines() {
            let line = line?;
            if let Some(captures) = log_pattern.captures(&line) {
                let entry = LogEntry {
                    timestamp: captures[1].to_string(),
                    level: captures[2].to_string(),
                    source: captures[3].to_string(),
                    message: captures[4].to_string(),
                };
                self.log_entries.push(entry);
            }
        }

        Ok(())
    }

    pub fn generate_summary(&self) -> LogSummary {
        let mut error_count = 0;
        let mut warning_count = 0;
        let mut info_count = 0;
        let mut source_distribution = HashMap::new();
        let mut recent_errors = Vec::new();

        for entry in &self.log_entries {
            match entry.level.as_str() {
                "ERROR" => {
                    error_count += 1;
                    if recent_errors.len() < 10 {
                        recent_errors.push(LogEntry {
                            timestamp: entry.timestamp.clone(),
                            level: entry.level.clone(),
                            message: entry.message.clone(),
                            source: entry.source.clone(),
                        });
                    }
                }
                "WARNING" => warning_count += 1,
                "INFO" => info_count += 1,
                _ => {}
            }

            *source_distribution.entry(entry.source.clone()).or_insert(0) += 1;
        }

        LogSummary {
            total_entries: self.log_entries.len(),
            error_count,
            warning_count,
            info_count,
            source_distribution,
            recent_errors,
        }
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.log_entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn filter_by_source(&self, source: &str) -> Vec<&LogEntry> {
        self.log_entries
            .iter()
            .filter(|entry| entry.source == source)
            .collect()
    }

    pub fn search_messages(&self, keyword: &str) -> Vec<&LogEntry> {
        self.log_entries
            .iter()
            .filter(|entry| entry.message.contains(keyword))
            .collect()
    }
}

impl LogSummary {
    pub fn print_summary(&self) {
        println!("Log Analysis Summary");
        println!("====================");
        println!("Total entries: {}", self.total_entries);
        println!("Errors: {}", self.error_count);
        println!("Warnings: {}", self.warning_count);
        println!("Info messages: {}", self.info_count);
        println!("\nSource distribution:");
        for (source, count) in &self.source_distribution {
            println!("  {}: {}", source, count);
        }
        
        if !self.recent_errors.is_empty() {
            println!("\nRecent errors:");
            for error in &self.recent_errors {
                println!("  [{}] {}: {}", error.timestamp, error.source, error.message);
            }
        }
    }
}use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum LogLevel {
    Error,
    Warning,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "error" => Some(LogLevel::Error),
            "warning" => Some(LogLevel::Warning),
            "info" => Some(LogLevel::Info),
            "debug" => Some(LogLevel::Debug),
            "trace" => Some(LogLevel::Trace),
            _ => None,
        }
    }
}

pub struct LogAnalyzer {
    counts: HashMap<LogLevel, usize>,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            counts: HashMap::new(),
        }
    }

    pub fn analyze_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            self.process_line(&line);
        }

        Ok(())
    }

    fn process_line(&mut self, line: &str) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            if let Some(level) = LogLevel::from_str(parts[2]) {
                *self.counts.entry(level).or_insert(0) += 1;
            }
        }
    }

    pub fn get_counts(&self) -> &HashMap<LogLevel, usize> {
        &self.counts
    }

    pub fn total_errors(&self) -> usize {
        self.counts.get(&LogLevel::Error).copied().unwrap_or(0)
    }

    pub fn print_summary(&self) {
        println!("Log Analysis Summary:");
        println!("====================");
        
        let levels = [
            LogLevel::Error,
            LogLevel::Warning,
            LogLevel::Info,
            LogLevel::Debug,
            LogLevel::Trace,
        ];

        for level in levels {
            if let Some(count) = self.counts.get(&level) {
                println!("{:?}: {}", level, count);
            }
        }
        
        println!("Total entries: {}", self.counts.values().sum::<usize>());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_analyzer_counts() {
        let mut analyzer = LogAnalyzer::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "2023-10-01 10:00:00 INFO Application started").unwrap();
        writeln!(temp_file, "2023-10-01 10:01:00 ERROR Database connection failed").unwrap();
        writeln!(temp_file, "2023-10-01 10:02:00 WARNING High memory usage").unwrap();
        writeln!(temp_file, "2023-10-01 10:03:00 INFO User login successful").unwrap();
        writeln!(temp_file, "2023-10-01 10:04:00 ERROR File not found").unwrap();
        
        analyzer.analyze_file(temp_file.path()).unwrap();
        
        assert_eq!(analyzer.total_errors(), 2);
        assert_eq!(analyzer.get_counts().get(&LogLevel::Info), Some(&2));
        assert_eq!(analyzer.get_counts().get(&LogLevel::Warning), Some(&1));
    }

    #[test]
    fn test_log_level_parsing() {
        assert_eq!(LogLevel::from_str("ERROR"), Some(LogLevel::Error));
        assert_eq!(LogLevel::from_str("Warning"), Some(LogLevel::Warning));
        assert_eq!(LogLevel::from_str("INFO"), Some(LogLevel::Info));
        assert_eq!(LogLevel::from_str("debug"), Some(LogLevel::Debug));
        assert_eq!(LogLevel::from_str("trace"), Some(LogLevel::Trace));
        assert_eq!(LogLevel::from_str("unknown"), None);
    }
}