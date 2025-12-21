use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub fields: HashMap<String, Value>,
}

pub fn parse_json_log_line(line: &str) -> Result<LogEntry, Box<dyn Error>> {
    let json_value: Value = serde_json::from_str(line)?;
    
    let timestamp = json_value["timestamp"]
        .as_str()
        .ok_or("Missing timestamp field")?
        .to_string();
    
    let level = json_value["level"]
        .as_str()
        .ok_or("Missing level field")?
        .to_string();
    
    let message = json_value["message"]
        .as_str()
        .ok_or("Missing message field")?
        .to_string();
    
    let mut fields = HashMap::new();
    if let Value::Object(obj) = &json_value {
        for (key, value) in obj {
            if !["timestamp", "level", "message"].contains(&key.as_str()) {
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

pub fn process_log_file(path: &str) -> Result<Vec<LogEntry>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();
    
    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        match parse_json_log_line(&line) {
            Ok(entry) => entries.push(entry),
            Err(e) => eprintln!("Error parsing line {}: {}", line_num + 1, e),
        }
    }
    
    Ok(entries)
}

pub fn filter_by_level(entries: &[LogEntry], level: &str) -> Vec<&LogEntry> {
    entries
        .iter()
        .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
        .collect()
}

pub fn extract_field_values(entries: &[LogEntry], field_name: &str) -> Vec<&Value> {
    entries
        .iter()
        .filter_map(|entry| entry.fields.get(field_name))
        .collect()
}