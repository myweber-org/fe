
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug)]
struct LogSummary {
    total_entries: usize,
    error_count: usize,
    warning_count: usize,
    info_count: usize,
    unique_messages: HashMap<String, usize>,
}

impl LogSummary {
    fn new() -> Self {
        LogSummary {
            total_entries: 0,
            error_count: 0,
            warning_count: 0,
            info_count: 0,
            unique_messages: HashMap::new(),
        }
    }

    fn update(&mut self, entry: &LogEntry) {
        self.total_entries += 1;
        
        match entry.level.as_str() {
            "ERROR" => self.error_count += 1,
            "WARNING" => self.warning_count += 1,
            "INFO" => self.info_count += 1,
            _ => (),
        }
        
        *self.unique_messages
            .entry(entry.message.clone())
            .or_insert(0) += 1;
    }
}

fn parse_log_file<P: AsRef<Path>>(path: P) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<LogEntry>(&line) {
            Ok(entry) => entries.push(entry),
            Err(e) => eprintln!("Failed to parse line: {}\nError: {}", line, e),
        }
    }

    Ok(entries)
}

fn filter_logs_by_level(entries: &[LogEntry], level: &str) -> Vec<LogEntry> {
    entries
        .iter()
        .filter(|entry| entry.level == level)
        .cloned()
        .collect()
}

fn generate_summary(entries: &[LogEntry]) -> LogSummary {
    let mut summary = LogSummary::new();
    
    for entry in entries {
        summary.update(entry);
    }
    
    summary
}

fn save_filtered_logs<P: AsRef<Path>>(
    entries: &[LogEntry],
    path: P,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(path)?;
    let mut writer = std::io::BufWriter::new(file);
    
    for entry in entries {
        let json = serde_json::to_string(entry)?;
        writeln!(writer, "{}", json)?;
    }
    
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log_file = "application.log";
    
    println!("Processing log file: {}", log_file);
    
    let entries = parse_log_file(log_file)?;
    println!("Total log entries parsed: {}", entries.len());
    
    let summary = generate_summary(&entries);
    println!("Log Summary:");
    println!("  Total entries: {}", summary.total_entries);
    println!("  Errors: {}", summary.error_count);
    println!("  Warnings: {}", summary.warning_count);
    println!("  Info messages: {}", summary.info_count);
    println!("  Unique message types: {}", summary.unique_messages.len());
    
    let error_logs = filter_logs_by_level(&entries, "ERROR");
    if !error_logs.is_empty() {
        println!("\nFound {} error logs:", error_logs.len());
        save_filtered_logs(&error_logs, "errors.json")?;
        println!("Error logs saved to errors.json");
    }
    
    let top_messages: Vec<(&String, &usize)> = summary
        .unique_messages
        .iter()
        .take(5)
        .collect();
    
    if !top_messages.is_empty() {
        println!("\nTop 5 most frequent messages:");
        for (message, count) in top_messages {
            println!("  {} ({} occurrences)", message, count);
        }
    }
    
    Ok(())
}