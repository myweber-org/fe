use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct LogFilter {
    pub min_level: Option<String>,
    pub contains_text: Option<String>,
}

impl LogFilter {
    pub fn new() -> Self {
        LogFilter {
            min_level: None,
            contains_text: None,
        }
    }

    pub fn with_min_level(mut self, level: &str) -> Self {
        self.min_level = Some(level.to_lowercase());
        self
    }

    pub fn with_text_filter(mut self, text: &str) -> Self {
        self.contains_text = Some(text.to_lowercase());
        self
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<String>, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);
        let mut results = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| format!("Failed to read line {}: {}", line_num + 1, e))?;
            
            if let Ok(json_value) = serde_json::from_str::<Value>(&line) {
                if self.matches_filter(&json_value) {
                    results.push(line);
                }
            }
        }

        Ok(results)
    }

    fn matches_filter(&self, json: &Value) -> bool {
        if let Some(min_level) = &self.min_level {
            if let Some(level) = json.get("level").and_then(|v| v.as_str()) {
                let level_lower = level.to_lowercase();
                let level_order = Self::level_order(&level_lower);
                let min_order = Self::level_order(min_level);
                
                if level_order < min_order {
                    return false;
                }
            }
        }

        if let Some(search_text) = &self.contains_text {
            let json_string = json.to_string().to_lowercase();
            if !json_string.contains(search_text) {
                return false;
            }
        }

        true
    }

    fn level_order(level: &str) -> u8 {
        match level {
            "debug" => 1,
            "info" => 2,
            "warn" => 3,
            "error" => 4,
            "critical" => 5,
            _ => 0,
        }
    }
}

pub fn filter_logs<P: AsRef<Path>>(
    path: P,
    min_level: Option<&str>,
    search_text: Option<&str>
) -> Result<Vec<String>, String> {
    let mut filter = LogFilter::new();
    
    if let Some(level) = min_level {
        filter = filter.with_min_level(level);
    }
    
    if let Some(text) = search_text {
        filter = filter.with_text_filter(text);
    }
    
    filter.process_file(path)
}use serde::{Deserialize, Serialize};
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

struct LogProcessor {
    entries: Vec<LogEntry>,
}

impl LogProcessor {
    fn new() -> Self {
        LogProcessor {
            entries: Vec::new(),
        }
    }

    fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let entry: LogEntry = serde_json::from_str(&line)?;
            self.entries.push(entry);
        }
        Ok(())
    }

    fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.eq_ignore_ascii_case(level))
            .collect()
    }

    fn group_by_service(&self) -> HashMap<String, Vec<&LogEntry>> {
        let mut groups = HashMap::new();
        for entry in &self.entries {
            groups
                .entry(entry.service.clone())
                .or_insert_with(Vec::new)
                .push(entry);
        }
        groups
    }

    fn count_by_level(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for entry in &self.entries {
            *counts.entry(entry.level.clone()).or_insert(0) += 1;
        }
        counts
    }

    fn search_messages(&self, keyword: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.message.contains(keyword))
            .collect()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut processor = LogProcessor::new();
    processor.load_from_file("logs.jsonl")?;

    println!("Total entries: {}", processor.entries.len());

    let error_logs = processor.filter_by_level("ERROR");
    println!("Error logs: {}", error_logs.len());

    let service_groups = processor.group_by_service();
    for (service, logs) in &service_groups {
        println!("Service '{}': {} logs", service, logs.len());
    }

    let level_counts = processor.count_by_level();
    for (level, count) in &level_counts {
        println!("Level {}: {} entries", level, count);
    }

    let search_results = processor.search_messages("timeout");
    println!("Found {} entries containing 'timeout'", search_results.len());

    Ok(())
}