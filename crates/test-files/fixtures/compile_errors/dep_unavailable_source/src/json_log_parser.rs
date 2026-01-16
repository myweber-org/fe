use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, PartialEq)]
enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    fn from_str(level: &str) -> Option<Self> {
        match level.to_lowercase().as_str() {
            "error" => Some(LogLevel::Error),
            "warn" => Some(LogLevel::Warn),
            "info" => Some(LogLevel::Info),
            "debug" => Some(LogLevel::Debug),
            "trace" => Some(LogLevel::Trace),
            _ => None,
        }
    }
}

struct LogParser {
    filters: HashMap<String, LogLevel>,
}

impl LogParser {
    fn new() -> Self {
        LogParser {
            filters: HashMap::new(),
        }
    }

    fn add_filter(&mut self, key: String, level: LogLevel) {
        self.filters.insert(key, level);
    }

    fn parse_file(&self, file_path: &str) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut filtered_logs = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(json_value) = serde_json::from_str::<Value>(&line) {
                if self.should_include(&json_value) {
                    filtered_logs.push(json_value);
                }
            }
        }

        Ok(filtered_logs)
    }

    fn should_include(&self, log_entry: &Value) -> bool {
        for (key, required_level) in &self.filters {
            if let Some(level_str) = log_entry.get(key).and_then(|v| v.as_str()) {
                if let Some(actual_level) = LogLevel::from_str(level_str) {
                    match required_level {
                        LogLevel::Error => {
                            if actual_level != LogLevel::Error {
                                return false;
                            }
                        }
                        LogLevel::Warn => {
                            if actual_level != LogLevel::Error && actual_level != LogLevel::Warn {
                                return false;
                            }
                        }
                        LogLevel::Info => {
                            if actual_level == LogLevel::Debug || actual_level == LogLevel::Trace {
                                return false;
                            }
                        }
                        LogLevel::Debug => {
                            if actual_level == LogLevel::Trace {
                                return false;
                            }
                        }
                        LogLevel::Trace => {}
                    }
                } else {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = LogParser::new();
    parser.add_filter("level".to_string(), LogLevel::Error);

    let logs = parser.parse_file("logs.jsonl")?;
    
    for log in logs {
        println!("{}", serde_json::to_string_pretty(&log)?);
    }

    Ok(())
}