use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub message: String,
    pub fields: HashMap<String, Value>,
}

pub struct LogParser {
    min_level: Option<String>,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            min_level: None,
            start_time: None,
            end_time: None,
        }
    }

    pub fn set_min_level(&mut self, level: &str) -> &mut Self {
        self.min_level = Some(level.to_lowercase());
        self
    }

    pub fn set_time_range(&mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> &mut Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
    }

    pub fn parse_file(&self, path: &str) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = self.parse_line(&line) {
                if self.filter_entry(&entry) {
                    entries.push(entry);
                }
            }
        }

        Ok(entries)
    }

    fn parse_line(&self, line: &str) -> Result<LogEntry, Box<dyn std::error::Error>> {
        let json_value: Value = serde_json::from_str(line)?;
        
        let timestamp_str = json_value["timestamp"]
            .as_str()
            .ok_or("Missing timestamp field")?;
        let timestamp = DateTime::parse_from_rfc3339(timestamp_str)?.with_timezone(&Utc);

        let level = json_value["level"]
            .as_str()
            .ok_or("Missing level field")?
            .to_string();

        let message = json_value["message"]
            .as_str()
            .ok_or("Missing message field")?
            .to_string();

        let mut fields = HashMap::new();
        if let Some(obj) = json_value.as_object() {
            for (key, value) in obj {
                if key != "timestamp" && key != "level" && key != "message" {
                    fields.insert(key.clone(), value.clone());
                }
            }
        }

        Ok(LogEntry {
            timestamp,
            level,
            message,
            fields,
        })
    }

    fn filter_entry(&self, entry: &LogEntry) -> bool {
        if let Some(min_level) = &self.min_level {
            let entry_level = entry.level.to_lowercase();
            let level_order = ["trace", "debug", "info", "warn", "error"];
            
            let min_idx = level_order.iter().position(|&l| l == min_level);
            let entry_idx = level_order.iter().position(|&l| l == entry_level);
            
            if let (Some(min_idx), Some(entry_idx)) = (min_idx, entry_idx) {
                if entry_idx < min_idx {
                    return false;
                }
            }
        }

        if let Some(start) = self.start_time {
            if entry.timestamp < start {
                return false;
            }
        }

        if let Some(end) = self.end_time {
            if entry.timestamp > end {
                return false;
            }
        }

        true
    }
}

impl Default for LogParser {
    fn default() -> Self {
        Self::new()
    }
}