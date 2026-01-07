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
}