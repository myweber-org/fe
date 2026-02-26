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
            } else if self.warning_pattern.is_match(&line) {
                *stats.get_mut("warnings").unwrap() += 1;
            } else if self.info_pattern.is_match(&line) {
                *stats.get_mut("info_messages").unwrap() += 1;
            }
        }
        
        Ok(stats)
    }

    pub fn generate_report(&self, stats: &HashMap<String, usize>) -> String {
        let total = stats.get("total_lines").unwrap_or(&0);
        let errors = stats.get("errors").unwrap_or(&0);
        let warnings = stats.get("warnings").unwrap_or(&0);
        let info = stats.get("info_messages").unwrap_or(&0);
        
        format!(
            "Log Analysis Report:\n\
            Total lines: {}\n\
            Errors: {}\n\
            Warnings: {}\n\
            Info messages: {}\n\
            Error rate: {:.2}%",
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
        writeln!(temp_file, "WARN: Disk space low").unwrap();
        writeln!(temp_file, "ERROR: Database connection failed").unwrap();
        writeln!(temp_file, "INFO: User login successful").unwrap();
        
        let stats = analyzer.analyze_log_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(stats.get("total_lines").unwrap(), &4);
        assert_eq!(stats.get("errors").unwrap(), &1);
        assert_eq!(stats.get("warnings").unwrap(), &1);
        assert_eq!(stats.get("info_messages").unwrap(), &2);
    }
}use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    component: String,
    message: String,
}

#[derive(Debug)]
pub struct LogStats {
    total_entries: usize,
    level_counts: HashMap<String, usize>,
    component_counts: HashMap<String, usize>,
    error_messages: Vec<String>,
}

pub struct LogAnalyzer {
    entries: Vec<LogEntry>,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if let Some(entry) = Self::parse_log_line(&line) {
                self.entries.push(entry);
            }
        }

        Ok(())
    }

    fn parse_log_line(line: &str) -> Option<LogEntry> {
        let parts: Vec<&str> = line.splitn(4, '|').collect();
        if parts.len() == 4 {
            Some(LogEntry {
                timestamp: parts[0].trim().to_string(),
                level: parts[1].trim().to_string(),
                component: parts[2].trim().to_string(),
                message: parts[3].trim().to_string(),
            })
        } else {
            None
        }
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .collect()
    }

    pub fn filter_by_component(&self, component: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.component.to_lowercase() == component.to_lowercase())
            .collect()
    }

    pub fn get_statistics(&self) -> LogStats {
        let mut level_counts = HashMap::new();
        let mut component_counts = HashMap::new();
        let mut error_messages = Vec::new();

        for entry in &self.entries {
            *level_counts.entry(entry.level.clone()).or_insert(0) += 1;
            *component_counts.entry(entry.component.clone()).or_insert(0) += 1;

            if entry.level.to_lowercase() == "error" {
                error_messages.push(entry.message.clone());
            }
        }

        LogStats {
            total_entries: self.entries.len(),
            level_counts,
            component_counts,
            error_messages,
        }
    }

    pub fn search_messages(&self, keyword: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.message.to_lowercase().contains(&keyword.to_lowercase()))
            .collect()
    }

    pub fn get_entries(&self) -> &[LogEntry] {
        &self.entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_log_line() {
        let line = "2023-10-01 12:00:00 | INFO | Network | Connection established";
        let entry = LogAnalyzer::parse_log_line(line).unwrap();

        assert_eq!(entry.timestamp, "2023-10-01 12:00:00");
        assert_eq!(entry.level, "INFO");
        assert_eq!(entry.component, "Network");
        assert_eq!(entry.message, "Connection established");
    }

    #[test]
    fn test_filter_by_level() {
        let mut analyzer = LogAnalyzer::new();
        analyzer.entries.push(LogEntry {
            timestamp: "2023-10-01 12:00:00".to_string(),
            level: "INFO".to_string(),
            component: "Network".to_string(),
            message: "Test".to_string(),
        });
        analyzer.entries.push(LogEntry {
            timestamp: "2023-10-01 12:01:00".to_string(),
            level: "ERROR".to_string(),
            component: "Database".to_string(),
            message: "Failed".to_string(),
        });

        let info_entries = analyzer.filter_by_level("INFO");
        assert_eq!(info_entries.len(), 1);
        assert_eq!(info_entries[0].level, "INFO");
    }
}