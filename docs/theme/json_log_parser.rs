use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
enum LogLevel {
    INFO,
    WARN,
    ERROR,
    DEBUG,
}

#[derive(Debug, Deserialize, Serialize)]
struct LogEntry {
    timestamp: String,
    level: LogLevel,
    message: String,
    component: String,
}

struct LogFilter {
    min_level: LogLevel,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
    component_filter: Option<String>,
}

impl LogFilter {
    fn new(min_level: LogLevel) -> Self {
        LogFilter {
            min_level,
            start_time: None,
            end_time: None,
            component_filter: None,
        }
    }

    fn with_time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
    }

    fn with_component(mut self, component: &str) -> Self {
        self.component_filter = Some(component.to_string());
        self
    }

    fn matches(&self, entry: &LogEntry) -> bool {
        if entry.level < self.min_level {
            return false;
        }

        if let Some(ref component) = self.component_filter {
            if entry.component != *component {
                return false;
            }
        }

        if let Ok(entry_time) = DateTime::parse_from_rfc3339(&entry.timestamp) {
            let entry_utc = entry_time.with_timezone(&Utc);
            
            if let Some(start) = self.start_time {
                if entry_utc < start {
                    return false;
                }
            }
            
            if let Some(end) = self.end_time {
                if entry_utc > end {
                    return false;
                }
            }
        }

        true
    }
}

fn parse_log_file(path: &str, filter: &LogFilter) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut filtered_entries = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<LogEntry>(&line) {
            Ok(entry) => {
                if filter.matches(&entry) {
                    filtered_entries.push(entry);
                }
            }
            Err(e) => eprintln!("Failed to parse log line: {} - {}", line, e),
        }
    }

    Ok(filtered_entries)
}

fn analyze_logs(entries: &[LogEntry]) {
    let mut level_counts = std::collections::HashMap::new();
    let mut component_counts = std::collections::HashMap::new();

    for entry in entries {
        *level_counts.entry(&entry.level).or_insert(0) += 1;
        *component_counts.entry(&entry.component).or_insert(0) += 1;
    }

    println!("Log Analysis:");
    println!("Total entries: {}", entries.len());
    println!("\nBy level:");
    for (level, count) in &level_counts {
        println!("  {:?}: {}", level, count);
    }
    println!("\nBy component:");
    for (component, count) in &component_counts {
        println!("  {}: {}", component, count);
    }
}

fn main() {
    let filter = LogFilter::new(LogLevel::WARN)
        .with_component("database");

    match parse_log_file("logs/app.log", &filter) {
        Ok(entries) => {
            println!("Found {} matching log entries", entries.len());
            analyze_logs(&entries);
            
            if let Some(first_error) = entries.iter().find(|e| e.level == LogLevel::ERROR) {
                println!("\nFirst ERROR entry:");
                println!("  Time: {}", first_error.timestamp);
                println!("  Component: {}", first_error.component);
                println!("  Message: {}", first_error.message);
            }
        }
        Err(e) => eprintln!("Error parsing logs: {}", e),
    }
}

impl PartialOrd for LogLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let self_val = match self {
            LogLevel::DEBUG => 0,
            LogLevel::INFO => 1,
            LogLevel::WARN => 2,
            LogLevel::ERROR => 3,
        };
        let other_val = match other {
            LogLevel::DEBUG => 0,
            LogLevel::INFO => 1,
            LogLevel::WARN => 2,
            LogLevel::ERROR => 3,
        };
        Some(self_val.cmp(&other_val))
    }
}