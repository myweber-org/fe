use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
struct LogEntry {
    timestamp: DateTime<Utc>,
    level: String,
    message: String,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

struct LogParser {
    min_level: Option<String>,
    search_term: Option<String>,
}

impl LogParser {
    fn new() -> Self {
        LogParser {
            min_level: None,
            search_term: None,
        }
    }

    fn with_min_level(mut self, level: &str) -> Self {
        self.min_level = Some(level.to_lowercase());
        self
    }

    fn with_search(mut self, term: &str) -> Self {
        self.search_term = Some(term.to_lowercase());
        self
    }

    fn parse_file(&self, path: &str) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = serde_json::from_str::<LogEntry>(&line) {
                if self.matches_filter(&entry) {
                    entries.push(entry);
                }
            }
        }

        Ok(entries)
    }

    fn matches_filter(&self, entry: &LogEntry) -> bool {
        if let Some(min_level) = &self.min_level {
            let entry_level = entry.level.to_lowercase();
            let level_priority = |l: &str| match l {
                "error" => 4,
                "warn" => 3,
                "info" => 2,
                "debug" => 1,
                _ => 0,
            };

            if level_priority(&entry_level) < level_priority(min_level) {
                return false;
            }
        }

        if let Some(term) = &self.search_term {
            if !entry.message.to_lowercase().contains(term) {
                return false;
            }
        }

        true
    }
}

fn format_entry(entry: &LogEntry) -> String {
    let extra_str = if entry.extra.is_empty() {
        String::new()
    } else {
        format!(" | {:?}", entry.extra)
    };
    
    format!(
        "[{}] {}: {}{}",
        entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
        entry.level.to_uppercase(),
        entry.message,
        extra_str
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parser = LogParser::new()
        .with_min_level("info")
        .with_search("connection");

    let entries = parser.parse_file("app.log")?;
    
    for entry in entries {
        println!("{}", format_entry(&entry));
    }

    Ok(())
}