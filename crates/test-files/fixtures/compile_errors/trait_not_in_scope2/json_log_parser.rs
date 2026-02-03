use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize, Serialize)]
struct LogEntry {
    timestamp: DateTime<Utc>,
    level: String,
    message: String,
    component: Option<String>,
    metadata: Option<serde_json::Value>,
}

#[derive(Debug)]
struct LogParser {
    min_level: String,
    filter_component: Option<String>,
}

impl LogParser {
    fn new(min_level: &str) -> Self {
        LogParser {
            min_level: min_level.to_lowercase(),
            filter_component: None,
        }
    }

    fn with_component_filter(mut self, component: &str) -> Self {
        self.filter_component = Some(component.to_string());
        self
    }

    fn parse_file(&self, path: &str) -> Result<Vec<LogEntry>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = self.parse_line(&line) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    fn parse_line(&self, line: &str) -> Result<LogEntry, Box<dyn Error>> {
        let entry: LogEntry = serde_json::from_str(line)?;
        
        if !self.matches_level(&entry.level) {
            return Err("Log level below threshold".into());
        }

        if let Some(ref filter) = self.filter_component {
            if let Some(ref component) = entry.component {
                if component != filter {
                    return Err("Component filter mismatch".into());
                }
            } else {
                return Err("No component specified".into());
            }
        }

        Ok(entry)
    }

    fn matches_level(&self, level: &str) -> bool {
        let level_order = ["trace", "debug", "info", "warn", "error"];
        let entry_level = level.to_lowercase();
        
        let min_index = level_order.iter()
            .position(|&l| l == self.min_level)
            .unwrap_or(0);
        
        let entry_index = level_order.iter()
            .position(|&l| l == entry_level)
            .unwrap_or(level_order.len() - 1);

        entry_index >= min_index
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let parser = LogParser::new("info")
        .with_component_filter("api");

    let entries = parser.parse_file("logs/app.log")?;
    
    for entry in entries {
        println!("{} [{:5}] {}",
            entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
            entry.level.to_uppercase(),
            entry.message
        );
    }

    Ok(())
}