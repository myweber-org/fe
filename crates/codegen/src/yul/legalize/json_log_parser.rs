use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use serde_json::Value;
use chrono::{DateTime, FixedOffset};

#[derive(Debug)]
pub struct LogEntry {
    timestamp: DateTime<FixedOffset>,
    level: String,
    message: String,
    fields: HashMap<String, Value>,
}

pub struct LogParser {
    entries: Vec<LogEntry>,
}

impl LogParser {
    pub fn new() -> Self {
        LogParser {
            entries: Vec::new(),
        }
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = self.parse_line(&line) {
                self.entries.push(entry);
            }
        }

        Ok(())
    }

    fn parse_line(&self, line: &str) -> Result<LogEntry, Box<dyn std::error::Error>> {
        let json_value: Value = serde_json::from_str(line)?;

        let timestamp_str = json_value["timestamp"]
            .as_str()
            .ok_or("Missing timestamp field")?;
        let timestamp = DateTime::parse_from_rfc3339(timestamp_str)?;

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

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .collect()
    }

    pub fn search_messages(&self, query: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.message.contains(query))
            .collect()
    }

    pub fn summarize(&self) -> HashMap<String, usize> {
        let mut summary = HashMap::new();
        for entry in &self.entries {
            *summary.entry(entry.level.clone()).or_insert(0) += 1;
        }
        summary
    }

    pub fn get_entries_in_time_range(
        &self,
        start: DateTime<FixedOffset>,
        end: DateTime<FixedOffset>,
    ) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.timestamp >= start && entry.timestamp <= end)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_parser_creation() {
        let parser = LogParser::new();
        assert_eq!(parser.entries.len(), 0);
    }

    #[test]
    fn test_summarize_empty() {
        let parser = LogParser::new();
        let summary = parser.summarize();
        assert!(summary.is_empty());
    }
}