
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
    extra_fields: HashMap<String, serde_json::Value>,
}

struct LogProcessor {
    min_level: String,
    include_fields: Vec<String>,
    exclude_fields: Vec<String>,
}

impl LogProcessor {
    fn new(min_level: &str) -> Self {
        LogProcessor {
            min_level: min_level.to_lowercase(),
            include_fields: Vec::new(),
            exclude_fields: Vec::new(),
        }
    }

    fn with_included_fields(mut self, fields: &[&str]) -> Self {
        self.include_fields = fields.iter().map(|s| s.to_string()).collect();
        self
    }

    fn with_excluded_fields(mut self, fields: &[&str]) -> Self {
        self.exclude_fields = fields.iter().map(|s| s.to_string()).collect();
        self
    }

    fn process_file(&self, file_path: &str) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut processed_logs = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(log_entry) = self.parse_line(&line) {
                if self.should_include(&log_entry) {
                    processed_logs.push(log_entry);
                }
            }
        }

        Ok(processed_logs)
    }

    fn parse_line(&self, line: &str) -> Result<LogEntry, Box<dyn std::error::Error>> {
        let mut entry: LogEntry = serde_json::from_str(line)?;
        
        if !self.include_fields.is_empty() {
            entry.extra_fields.retain(|k, _| self.include_fields.contains(k));
        }
        
        if !self.exclude_fields.is_empty() {
            entry.extra_fields.retain(|k, _| !self.exclude_fields.contains(k));
        }

        Ok(entry)
    }

    fn should_include(&self, entry: &LogEntry) -> bool {
        let level_order = |level: &str| -> u8 {
            match level.to_lowercase().as_str() {
                "trace" => 1,
                "debug" => 2,
                "info" => 3,
                "warn" => 4,
                "error" => 5,
                "fatal" => 6,
                _ => 0,
            }
        };

        level_order(&entry.level) >= level_order(&self.min_level)
    }

    fn format_output(&self, entries: &[LogEntry]) -> String {
        let mut output = String::new();
        
        for entry in entries {
            output.push_str(&format!(
                "[{}] {}: {}\n",
                entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
                entry.level.to_uppercase(),
                entry.message
            ));

            if !entry.extra_fields.is_empty() {
                for (key, value) in &entry.extra_fields {
                    output.push_str(&format!("  {}: {}\n", key, value));
                }
            }
            output.push('\n');
        }

        output
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let processor = LogProcessor::new("info")
        .with_included_fields(&["user_id", "request_id", "duration_ms"])
        .with_excluded_fields(&["internal_trace", "sensitive_data"]);

    let logs = processor.process_file("application.log")?;
    
    if !logs.is_empty() {
        println!("{}", processor.format_output(&logs));
        println!("Processed {} log entries", logs.len());
    } else {
        println!("No matching log entries found");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_parsing() {
        let log_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"info","message":"User login","user_id":123,"request_id":"abc-123"}"#;
        
        let processor = LogProcessor::new("info");
        let entry = processor.parse_line(log_data).unwrap();
        
        assert_eq!(entry.level, "info");
        assert_eq!(entry.message, "User login");
        assert_eq!(entry.extra_fields.get("user_id").unwrap().as_i64().unwrap(), 123);
    }

    #[test]
    fn test_level_filtering() {
        let processor = LogProcessor::new("warn");
        
        let debug_entry = LogEntry {
            timestamp: Utc::now(),
            level: "debug".to_string(),
            message: "Test debug".to_string(),
            extra_fields: HashMap::new(),
        };
        
        let warn_entry = LogEntry {
            timestamp: Utc::now(),
            level: "warn".to_string(),
            message: "Test warning".to_string(),
            extra_fields: HashMap::new(),
        };
        
        assert!(!processor.should_include(&debug_entry));
        assert!(processor.should_include(&warn_entry));
    }

    #[test]
    fn test_file_processing() -> Result<(), Box<dyn std::error::Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, r#"{{"timestamp":"2024-01-15T10:30:00Z","level":"error","message":"Database connection failed","error_code":500}}"#)?;
        writeln!(temp_file, r#"{{"timestamp":"2024-01-15T10:31:00Z","level":"debug","message":"Query executed","duration_ms":45}}"#)?;
        
        let processor = LogProcessor::new("error");
        let logs = processor.process_file(temp_file.path().to_str().unwrap())?;
        
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].level, "error");
        
        Ok(())
    }
}