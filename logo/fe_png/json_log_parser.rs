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
enum ParseError {
    IoError(std::io::Error),
    JsonError(serde_json::Error),
    InvalidLogFormat(String),
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::IoError(err)
    }
}

impl From<serde_json::Error> for ParseError {
    fn from(err: serde_json::Error) -> Self {
        ParseError::JsonError(err)
    }
}

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

    fn parse_file(&self, path: &str) -> Result<Vec<LogEntry>, ParseError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line_content = line?;
            match self.parse_line(&line_content) {
                Ok(Some(entry)) => entries.push(entry),
                Ok(None) => continue,
                Err(e) => eprintln!("Warning: Failed to parse line {}: {}", line_num + 1, e),
            }
        }

        Ok(entries)
    }

    fn parse_line(&self, line: &str) -> Result<Option<LogEntry>, ParseError> {
        let entry: LogEntry = serde_json::from_str(line)?;
        
        if !self.matches_level(&entry.level) {
            return Ok(None);
        }

        if let Some(ref filter) = self.filter_component {
            if let Some(ref component) = entry.component {
                if component != filter {
                    return Ok(None);
                }
            } else {
                return Ok(None);
            }
        }

        Ok(Some(entry))
    }

    fn matches_level(&self, level: &str) -> bool {
        let level_order = ["trace", "debug", "info", "warn", "error", "fatal"];
        let min_index = level_order.iter()
            .position(|&l| l == self.min_level)
            .unwrap_or(0);
        let entry_index = level_order.iter()
            .position(|&l| l == level.to_lowercase())
            .unwrap_or(level_order.len());
        
        entry_index >= min_index
    }
}

fn analyze_logs(entries: &[LogEntry]) {
    let mut level_counts = std::collections::HashMap::new();
    let mut component_counts = std::collections::HashMap::new();

    for entry in entries {
        *level_counts.entry(entry.level.clone()).or_insert(0) += 1;
        if let Some(component) = &entry.component {
            *component_counts.entry(component.clone()).or_insert(0) += 1;
        }
    }

    println!("Log Analysis:");
    println!("Total entries: {}", entries.len());
    println!("\nLevel distribution:");
    for (level, count) in &level_counts {
        println!("  {}: {}", level, count);
    }
    
    if !component_counts.is_empty() {
        println!("\nComponent distribution:");
        for (component, count) in &component_counts {
            println!("  {}: {}", component, count);
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let parser = LogParser::new("info")
        .with_component_filter("api");

    match parser.parse_file("application.log") {
        Ok(entries) => {
            println!("Successfully parsed {} log entries", entries.len());
            analyze_logs(&entries);
            
            if let Some(latest) = entries.last() {
                println!("\nLatest log entry:");
                println!("  Time: {}", latest.timestamp);
                println!("  Level: {}", latest.level);
                println!("  Message: {}", latest.message);
            }
        }
        Err(ParseError::IoError(e)) if e.kind() == std::io::ErrorKind::NotFound => {
            eprintln!("Log file not found. Creating sample data...");
            create_sample_log()?;
        }
        Err(e) => return Err(Box::new(e)),
    }

    Ok(())
}

fn create_sample_log() -> Result<(), Box<dyn Error>> {
    use std::io::Write;
    
    let sample_entries = vec![
        LogEntry {
            timestamp: Utc::now(),
            level: "INFO".to_string(),
            message: "Application started".to_string(),
            component: Some("system".to_string()),
            metadata: Some(serde_json::json!({"pid": 1234})),
        },
        LogEntry {
            timestamp: Utc::now(),
            level: "DEBUG".to_string(),
            message: "Initializing modules".to_string(),
            component: Some("api".to_string()),
            metadata: None,
        },
        LogEntry {
            timestamp: Utc::now(),
            level: "ERROR".to_string(),
            message: "Failed to connect to database".to_string(),
            component: Some("database".to_string()),
            metadata: Some(serde_json::json!({"attempt": 3})),
        },
    ];

    let mut file = File::create("application.log")?;
    for entry in sample_entries {
        let json = serde_json::to_string(&entry)?;
        writeln!(file, "{}", json)?;
    }

    println!("Created sample log file with 3 entries");
    Ok(())
}