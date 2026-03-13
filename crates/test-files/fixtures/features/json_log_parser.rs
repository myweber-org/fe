use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    service: String,
    message: String,
    metadata: HashMap<String, String>,
}

struct LogParser {
    entries: Vec<LogEntry>,
}

impl LogParser {
    fn new() -> Self {
        LogParser { entries: Vec::new() }
    }

    fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<LogEntry>(&line) {
                Ok(entry) => self.entries.push(entry),
                Err(e) => eprintln!("Failed to parse line: {} - {}", line, e),
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

    fn filter_by_service(&self, service: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.service == service)
            .collect()
    }

    fn count_by_level(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for entry in &self.entries {
            *counts.entry(entry.level.clone()).or_insert(0) += 1;
        }
        counts
    }

    fn count_by_service(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for entry in &self.entries {
            *counts.entry(entry.service.clone()).or_insert(0) += 1;
        }
        counts
    }

    fn search_messages(&self, keyword: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.message.to_lowercase().contains(&keyword.to_lowercase()))
            .collect()
    }

    fn get_time_range(&self) -> Option<(String, String)> {
        if self.entries.is_empty() {
            return None;
        }

        let mut timestamps: Vec<&String> = self.entries.iter().map(|e| &e.timestamp).collect();
        timestamps.sort();

        Some((timestamps[0].clone(), timestamps[timestamps.len() - 1].clone()))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = LogParser::new();
    
    parser.load_from_file("logs.jsonl")?;
    
    println!("Total entries: {}", parser.entries.len());
    
    let error_logs = parser.filter_by_level("ERROR");
    println!("Error logs: {}", error_logs.len());
    
    let level_counts = parser.count_by_level();
    println!("Log level distribution:");
    for (level, count) in &level_counts {
        println!("  {}: {}", level, count);
    }
    
    if let Some((start, end)) = parser.get_time_range() {
        println!("Time range: {} - {}", start, end);
    }
    
    let search_results = parser.search_messages("timeout");
    println!("Found {} entries containing 'timeout'", search_results.len());
    
    Ok(())
}