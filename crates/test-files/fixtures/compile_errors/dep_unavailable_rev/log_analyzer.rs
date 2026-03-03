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

    pub fn load_from_file(&mut self, path: &str) -> Result<(), std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let log_pattern = Regex::new(r"(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}) \[(\w+)\] (.+)").unwrap();

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

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn get_summary(&self) -> HashMap<String, usize> {
        self.level_counts.clone()
    }

    pub fn search_messages(&self, pattern: &str) -> Vec<&LogEntry> {
        let search_regex = Regex::new(pattern).unwrap_or_else(|_| Regex::new(".*").unwrap());
        self.entries
            .iter()
            .filter(|entry| search_regex.is_match(&entry.message))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_analyzer_creation() {
        let analyzer = LogAnalyzer::new();
        assert_eq!(analyzer.entries.len(), 0);
    }

    #[test]
    fn test_filter_by_level() {
        let mut analyzer = LogAnalyzer::new();
        analyzer.entries.push(LogEntry {
            timestamp: "2024-01-01 10:00:00".to_string(),
            level: "ERROR".to_string(),
            message: "Test error".to_string(),
        });
        
        let errors = analyzer.filter_by_level("ERROR");
        assert_eq!(errors.len(), 1);
    }
}use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use chrono::{DateTime, Utc};
use regex::Regex;

#[derive(Debug)]
pub struct LogEntry {
    timestamp: DateTime<Utc>,
    level: String,
    component: String,
    message: String,
}

pub struct LogAnalyzer {
    entries: Vec<LogEntry>,
    error_count: usize,
    warning_count: usize,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
            error_count: 0,
            warning_count: 0,
        }
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let log_pattern = Regex::new(r"\[(?P<timestamp>[\d\-:T.]+Z)\] \[(?P<level>\w+)\] \[(?P<component>[\w\.]+)\]: (?P<message>.+)")?;

        for line in reader.lines() {
            let line = line?;
            if let Some(captures) = log_pattern.captures(&line) {
                let timestamp_str = captures.name("timestamp").unwrap().as_str();
                let timestamp = DateTime::parse_from_rfc3339(timestamp_str)?.with_timezone(&Utc);
                let level = captures.name("level").unwrap().as_str().to_string();
                let component = captures.name("component").unwrap().as_str().to_string();
                let message = captures.name("message").unwrap().as_str().to_string();

                match level.as_str() {
                    "ERROR" => self.error_count += 1,
                    "WARNING" => self.warning_count += 1,
                    _ => {}
                }

                self.entries.push(LogEntry {
                    timestamp,
                    level,
                    component,
                    message,
                });
            }
        }

        Ok(())
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn get_component_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        for entry in &self.entries {
            *stats.entry(entry.component.clone()).or_insert(0) += 1;
        }
        stats
    }

    pub fn get_error_count(&self) -> usize {
        self.error_count
    }

    pub fn get_warning_count(&self) -> usize {
        self.warning_count
    }

    pub fn get_total_entries(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_analysis() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let log_data = "[2023-10-05T14:30:00Z] [INFO] [app.core]: Application started\n\
                       [2023-10-05T14:31:00Z] [WARNING] [app.network]: Connection timeout\n\
                       [2023-10-05T14:32:00Z] [ERROR] [app.database]: Failed to connect to database\n";
        
        write!(temp_file, "{}", log_data).unwrap();
        
        let mut analyzer = LogAnalyzer::new();
        analyzer.load_from_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(analyzer.get_total_entries(), 3);
        assert_eq!(analyzer.get_error_count(), 1);
        assert_eq!(analyzer.get_warning_count(), 1);
        
        let errors = analyzer.filter_by_level("ERROR");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].component, "app.database");
        
        let stats = analyzer.get_component_stats();
        assert_eq!(stats.get("app.core"), Some(&1));
        assert_eq!(stats.get("app.network"), Some(&1));
        assert_eq!(stats.get("app.database"), Some(&1));
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

    pub fn analyze_file(&self, file_path: &str) -> Result<HashMap<String, usize>, std::io::Error> {
        let file = File::open(file_path)?;
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
            *stats.entry("ERROR".to_string()).or_insert(0) += 1;
        } else if self.warn_pattern.is_match(line) {
            *stats.entry("WARN".to_string()).or_insert(0) += 1;
        } else if self.info_pattern.is_match(line) {
            *stats.entry("INFO".to_string()).or_insert(0) += 1;
        }
    }

    pub fn print_summary(&self, stats: &HashMap<String, usize>) {
        println!("Log Level Summary:");
        for (level, count) in stats {
            println!("{}: {}", level, count);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_analysis() {
        let analyzer = LogAnalyzer::new();
        let test_log = "INFO: Application started\nERROR: Database connection failed\nWARN: High memory usage\nINFO: Request processed";
        
        let mut stats = HashMap::new();
        for line in test_log.lines() {
            analyzer.process_line(line, &mut stats);
        }

        assert_eq!(stats.get("INFO"), Some(&2));
        assert_eq!(stats.get("ERROR"), Some(&1));
        assert_eq!(stats.get("WARN"), Some(&1));
    }
}