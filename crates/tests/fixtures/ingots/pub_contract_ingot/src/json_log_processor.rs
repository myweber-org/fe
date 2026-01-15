
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use serde_json::Value;
use chrono::{DateTime, FixedOffset};

#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: DateTime<FixedOffset>,
    pub level: String,
    pub message: String,
    pub fields: HashMap<String, Value>,
}

pub struct LogProcessor {
    pub entries: Vec<LogEntry>,
    pub error_count: usize,
    pub warning_count: usize,
}

impl LogProcessor {
    pub fn new() -> Self {
        LogProcessor {
            entries: Vec::new(),
            error_count: 0,
            warning_count: 0,
        }
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = self.parse_log_line(&line) {
                self.entries.push(entry);
            }
        }

        self.update_counts();
        Ok(())
    }

    fn parse_log_line(&self, line: &str) -> Result<LogEntry, Box<dyn std::error::Error>> {
        let json_value: Value = serde_json::from_str(line)?;

        let timestamp_str = json_value["timestamp"]
            .as_str()
            .ok_or("Missing timestamp field")?;
        let timestamp = DateTime::parse_from_rfc3339(timestamp_str)?;

        let level = json_value["level"]
            .as_str()
            .unwrap_or("INFO")
            .to_string();

        let message = json_value["message"]
            .as_str()
            .unwrap_or("")
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

    fn update_counts(&mut self) {
        self.error_count = self.entries.iter()
            .filter(|e| e.level == "ERROR")
            .count();
        
        self.warning_count = self.entries.iter()
            .filter(|e| e.level == "WARN")
            .count();
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries.iter()
            .filter(|e| e.level == level)
            .collect()
    }

    pub fn get_time_range(&self) -> Option<(DateTime<FixedOffset>, DateTime<FixedOffset>)> {
        if self.entries.is_empty() {
            return None;
        }

        let mut min_time = &self.entries[0].timestamp;
        let mut max_time = &self.entries[0].timestamp;

        for entry in &self.entries {
            if entry.timestamp < *min_time {
                min_time = &entry.timestamp;
            }
            if entry.timestamp > *max_time {
                max_time = &entry.timestamp;
            }
        }

        Some((*min_time, *max_time))
    }

    pub fn summarize(&self) -> String {
        let time_range = self.get_time_range();
        let time_range_str = match time_range {
            Some((start, end)) => format!("{} to {}", start, end),
            None => "No entries".to_string(),
        };

        format!(
            "Log Summary:\n\
             Total entries: {}\n\
             Errors: {}\n\
             Warnings: {}\n\
             Time range: {}\n\
             Unique field keys: {}",
            self.entries.len(),
            self.error_count,
            self.warning_count,
            time_range_str,
            self.get_unique_field_count()
        )
    }

    fn get_unique_field_count(&self) -> usize {
        let mut unique_keys = std::collections::HashSet::new();
        for entry in &self.entries {
            for key in entry.fields.keys() {
                unique_keys.insert(key);
            }
        }
        unique_keys.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_parsing() {
        let mut processor = LogProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{"timestamp":"2024-01-15T10:30:00+00:00","level":"ERROR","message":"Database connection failed","error_code":500}}"#).unwrap();
        writeln!(temp_file, r#"{{"timestamp":"2024-01-15T10:31:00+00:00","level":"INFO","message":"Service started","port":8080}}"#).unwrap();
        
        let result = processor.load_from_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.entries.len(), 2);
        assert_eq!(processor.error_count, 1);
        assert_eq!(processor.warning_count, 0);
    }

    #[test]
    fn test_filter_by_level() {
        let mut processor = LogProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{"timestamp":"2024-01-15T10:30:00+00:00","level":"ERROR","message":"Error 1"}}"#).unwrap();
        writeln!(temp_file, r#"{{"timestamp":"2024-01-15T10:31:00+00:00","level":"INFO","message":"Info 1"}}"#).unwrap();
        writeln!(temp_file, r#"{{"timestamp":"2024-01-15T10:32:00+00:00","level":"ERROR","message":"Error 2"}}"#).unwrap();
        
        processor.load_from_file(temp_file.path().to_str().unwrap()).unwrap();
        
        let errors = processor.filter_by_level("ERROR");
        assert_eq!(errors.len(), 2);
        
        let infos = processor.filter_by_level("INFO");
        assert_eq!(infos.len(), 1);
    }
}