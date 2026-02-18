use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize, Serialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    service: String,
}

#[derive(Debug)]
struct LogProcessor {
    entries: Vec<LogEntry>,
}

impl LogProcessor {
    fn new() -> Self {
        LogProcessor {
            entries: Vec::new(),
        }
    }

    fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<LogEntry>(&line) {
                Ok(entry) => self.entries.push(entry),
                Err(e) => eprintln!("Failed to parse line: {}. Error: {}", line, e),
            }
        }

        Ok(())
    }

    fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .collect()
    }

    fn filter_by_timestamp_range(
        &self,
        start: &str,
        end: &str,
    ) -> Result<Vec<&LogEntry>, Box<dyn Error>> {
        let start_time: DateTime<Utc> = start.parse()?;
        let end_time: DateTime<Utc> = end.parse()?;

        let filtered: Vec<&LogEntry> = self
            .entries
            .iter()
            .filter(|entry| {
                if let Ok(entry_time) = entry.timestamp.parse::<DateTime<Utc>>() {
                    entry_time >= start_time && entry_time <= end_time
                } else {
                    false
                }
            })
            .collect();

        Ok(filtered)
    }

    fn count_by_service(&self) -> std::collections::HashMap<String, usize> {
        let mut counts = std::collections::HashMap::new();
        for entry in &self.entries {
            *counts.entry(entry.service.clone()).or_insert(0) += 1;
        }
        counts
    }

    fn save_filtered(&self, entries: Vec<&LogEntry>, output_path: &str) -> Result<(), Box<dyn Error>> {
        let mut writer = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(output_path)?;

        for entry in entries {
            let json = serde_json::to_string(entry)?;
            writeln!(&mut writer, "{}", json)?;
        }

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut processor = LogProcessor::new();
    
    processor.load_from_file("logs.jsonl")?;
    
    println!("Total entries loaded: {}", processor.entries.len());
    
    let error_logs = processor.filter_by_level("error");
    println!("Error logs count: {}", error_logs.len());
    
    let service_counts = processor.count_by_service();
    for (service, count) in service_counts {
        println!("Service '{}': {} logs", service, count);
    }
    
    if let Ok(recent_logs) = processor.filter_by_timestamp_range(
        "2024-01-01T00:00:00Z",
        "2024-12-31T23:59:59Z"
    ) {
        println!("Logs in date range: {}", recent_logs.len());
        processor.save_filtered(recent_logs, "filtered_logs.jsonl")?;
    }
    
    Ok(())
}