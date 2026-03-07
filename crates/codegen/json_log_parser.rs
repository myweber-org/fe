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

struct LogFilter {
    min_level: Option<String>,
    contains_text: Option<String>,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
}

impl LogFilter {
    fn matches(&self, entry: &LogEntry) -> bool {
        if let Some(min_level) = &self.min_level {
            let levels = ["trace", "debug", "info", "warn", "error"];
            let entry_idx = levels.iter().position(|&l| l == entry.level.to_lowercase());
            let min_idx = levels.iter().position(|&l| l == min_level.to_lowercase());
            
            if let (Some(e_idx), Some(m_idx)) = (entry_idx, min_idx) {
                if e_idx < m_idx {
                    return false;
                }
            }
        }

        if let Some(text) = &self.contains_text {
            if !entry.message.contains(text) {
                return false;
            }
        }

        if let Some(start) = &self.start_time {
            if &entry.timestamp < start {
                return false;
            }
        }

        if let Some(end) = &self.end_time {
            if &entry.timestamp > end {
                return false;
            }
        }

        true
    }
}

fn parse_log_file(path: &str, filter: &LogFilter) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<LogEntry>(&line) {
            Ok(entry) => {
                if filter.matches(&entry) {
                    entries.push(entry);
                }
            }
            Err(e) => {
                eprintln!("Failed to parse line: {}, error: {}", line, e);
            }
        }
    }

    Ok(entries)
}

fn format_entry(entry: &LogEntry, show_extra: bool) -> String {
    let mut output = format!(
        "[{}] {}: {}",
        entry.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
        entry.level.to_uppercase(),
        entry.message
    );

    if show_extra && !entry.extra.is_empty() {
        output.push_str(&format!(" | Extra: {:?}", entry.extra));
    }

    output
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filter = LogFilter {
        min_level: Some("info".to_string()),
        contains_text: Some("request".to_string()),
        start_time: None,
        end_time: None,
    };

    let entries = parse_log_file("application.log", &filter)?;
    
    for entry in entries {
        println!("{}", format_entry(&entry, true));
    }

    println!("Total matching entries: {}", entries.len());
    Ok(())
}use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub fields: HashMap<String, Value>,
}

pub struct LogParser {
    min_level: String,
    required_fields: Vec<String>,
}

impl LogParser {
    pub fn new(min_level: &str) -> Self {
        LogParser {
            min_level: min_level.to_lowercase(),
            required_fields: Vec::new(),
        }
    }

    pub fn add_required_field(&mut self, field: &str) {
        self.required_fields.push(field.to_string());
    }

    pub fn parse_file(&self, path: &str) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
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

    fn parse_line(&self, line: &str) -> Result<LogEntry, Box<dyn std::error::Error>> {
        let json: Value = serde_json::from_str(line)?;
        
        let level = json.get("level")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_lowercase();

        if !self.is_level_allowed(&level) {
            return Err("Log level below threshold".into());
        }

        let timestamp = json.get("timestamp")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let message = json.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let mut fields = HashMap::new();
        if let Some(obj) = json.as_object() {
            for (key, value) in obj {
                if key != "timestamp" && key != "level" && key != "message" {
                    if self.required_fields.is_empty() || self.required_fields.contains(key) {
                        fields.insert(key.clone(), value.clone());
                    }
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

    fn is_level_allowed(&self, level: &str) -> bool {
        let level_order = vec!["trace", "debug", "info", "warn", "error", "fatal"];
        let min_index = level_order.iter().position(|&l| l == self.min_level);
        let current_index = level_order.iter().position(|&l| l == level);
        
        match (min_index, current_index) {
            (Some(min), Some(current)) => current >= min,
            _ => true,
        }
    }
}

pub fn filter_entries_by_field(
    entries: &[LogEntry],
    field_name: &str,
    field_value: &Value,
) -> Vec<&LogEntry> {
    entries
        .iter()
        .filter(|entry| entry.fields.get(field_name) == Some(field_value))
        .collect()
}