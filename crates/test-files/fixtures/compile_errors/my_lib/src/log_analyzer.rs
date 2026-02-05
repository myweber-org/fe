use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    component: String,
    message: String,
}

#[derive(Debug)]
pub struct LogSummary {
    total_entries: usize,
    error_count: usize,
    warning_count: usize,
    component_counts: HashMap<String, usize>,
    level_distribution: HashMap<String, usize>,
}

pub struct LogAnalyzer {
    pattern: Regex,
}

impl LogAnalyzer {
    pub fn new() -> Result<Self, regex::Error> {
        let pattern = Regex::new(r"\[(?P<timestamp>[^\]]+)\] \[(?P<level>\w+)\] \[(?P<component>[^\]]+)\] (?P<message>.+)")?;
        Ok(LogAnalyzer { pattern })
    }

    pub fn parse_line(&self, line: &str) -> Option<LogEntry> {
        self.pattern.captures(line).map(|caps| LogEntry {
            timestamp: caps["timestamp"].to_string(),
            level: caps["level"].to_string(),
            component: caps["component"].to_string(),
            message: caps["message"].to_string(),
        })
    }

    pub fn analyze_file<P: AsRef<Path>>(&self, path: P) -> std::io::Result<LogSummary> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut summary = LogSummary {
            total_entries: 0,
            error_count: 0,
            warning_count: 0,
            component_counts: HashMap::new(),
            level_distribution: HashMap::new(),
        };

        for line in reader.lines() {
            let line = line?;
            if let Some(entry) = self.parse_line(&line) {
                summary.total_entries += 1;
                
                *summary.level_distribution.entry(entry.level.clone()).or_insert(0) += 1;
                *summary.component_counts.entry(entry.component.clone()).or_insert(0) += 1;
                
                match entry.level.as_str() {
                    "ERROR" => summary.error_count += 1,
                    "WARN" => summary.warning_count += 1,
                    _ => {}
                }
            }
        }

        Ok(summary)
    }

    pub fn find_errors<P: AsRef<Path>>(&self, path: P) -> std::io::Result<Vec<LogEntry>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut errors = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Some(entry) = self.parse_line(&line) {
                if entry.level == "ERROR" {
                    errors.push(entry);
                }
            }
        }

        Ok(errors)
    }
}

impl LogSummary {
    pub fn print_report(&self) {
        println!("Log Analysis Report");
        println!("===================");
        println!("Total entries: {}", self.total_entries);
        println!("Errors: {}", self.error_count);
        println!("Warnings: {}", self.warning_count);
        
        println!("\nLevel Distribution:");
        for (level, count) in &self.level_distribution {
            println!("  {}: {}", level, count);
        }
        
        println!("\nComponent Activity:");
        for (component, count) in &self.component_counts {
            println!("  {}: {}", component, count);
        }
    }
}