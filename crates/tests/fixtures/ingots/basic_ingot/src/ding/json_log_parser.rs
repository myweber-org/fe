use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    #[serde(default)]
    error: Option<String>,
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
            Err(e) => eprintln!("Failed to parse line: {} - {}", line, e),
        }
    }

    Ok(entries)
}

fn filter_errors(entries: Vec<LogEntry>) -> Vec<LogEntry> {
    entries
        .into_iter()
        .filter(|entry| entry.level == "ERROR")
        .collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let entries = parse_log_file("application.log")?;
    let error_entries = filter_errors(entries);

    println!("Found {} error entries:", error_entries.len());
    for entry in error_entries {
        println!(
            "[{}] {} - {}",
            entry.timestamp,
            entry.level,
            entry.error.unwrap_or(entry.message)
        );
    }

    Ok(())
}