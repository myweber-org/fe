
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Deserialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    #[serde(flatten)]
    extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug)]
struct FilteredLogs {
    entries: Vec<LogEntry>,
    total_processed: usize,
}

fn parse_log_file(path: &str, min_level: &str, start_time: Option<DateTime<Utc>>) -> Result<FilteredLogs, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();
    let mut total_processed = 0;

    let level_order = vec!["trace", "debug", "info", "warn", "error", "fatal"];

    let min_level_index = level_order
        .iter()
        .position(|&l| l == min_level.to_lowercase())
        .unwrap_or(0);

    for line in reader.lines() {
        total_processed += 1;
        let line = line?;
        
        if line.trim().is_empty() {
            continue;
        }

        let entry: LogEntry = match serde_json::from_str(&line) {
            Ok(e) => e,
            Err(_) => continue,
        };

        let entry_level_index = level_order
            .iter()
            .position(|&l| l == entry.level.to_lowercase())
            .unwrap_or(0);

        if entry_level_index < min_level_index {
            continue;
        }

        if let Some(start) = start_time {
            let entry_time: DateTime<Utc> = entry.timestamp.parse().unwrap_or(Utc::now());
            if entry_time < start {
                continue;
            }
        }

        entries.push(entry);
    }

    Ok(FilteredLogs {
        entries,
        total_processed,
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    let logs = parse_log_file("application.log", "info", None)?;
    
    println!("Processed {} lines, found {} matching entries", 
             logs.total_processed, 
             logs.entries.len());
    
    for entry in logs.entries.iter().take(5) {
        println!("[{}] {}: {}", 
                 entry.timestamp, 
                 entry.level.to_uppercase(), 
                 entry.message);
    }
    
    Ok(())
}