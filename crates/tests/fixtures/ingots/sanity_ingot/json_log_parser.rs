use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use serde_json::Value;

#[derive(Debug)]
pub struct LogEntry {
    timestamp: String,
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
            if let Ok(json_value) = serde_json::from_str::<Value>(&line) {
                let entry = LogEntry {
                    timestamp: json_value["timestamp"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                    level: json_value["level"]
                        .as_str()
                        .unwrap_or("INFO")
                        .to_string(),
                    message: json_value["message"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                    fields: Self::extract_fields(&json_value),
                };
                self.entries.push(entry);
            }
        }
        Ok(())
    }

    fn extract_fields(json: &Value) -> HashMap<String, Value> {
        let mut fields = HashMap::new();
        if let Some(obj) = json.as_object() {
            for (key, value) in obj {
                if !["timestamp", "level", "message"].contains(&key.as_str()) {
                    fields.insert(key.clone(), value.clone());
                }
            }
        }
        fields
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn count_by_level(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for entry in &self.entries {
            *counts.entry(entry.level.clone()).or_insert(0) += 1;
        }
        counts
    }

    pub fn search_messages(&self, keyword: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.message.contains(keyword))
            .collect()
    }

    pub fn get_summary(&self) -> String {
        let total = self.entries.len();
        let counts = self.count_by_level();
        let mut summary = format!("Total entries: {}\n", total);
        for (level, count) in counts {
            summary.push_str(&format!("{}: {}\n", level, count));
        }
        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parser() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let log_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"ERROR","message":"Database connection failed","service":"api","error_code":500}
{"timestamp":"2024-01-15T10:31:00Z","level":"INFO","message":"Request processed","service":"web","duration_ms":45}
{"timestamp":"2024-01-15T10:32:00Z","level":"WARN","message":"High memory usage","service":"cache","memory_mb":2048}"#;
        write!(temp_file, "{}", log_data).unwrap();

        let mut parser = LogParser::new();
        parser.load_from_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(parser.entries.len(), 3);
        assert_eq!(parser.filter_by_level("ERROR").len(), 1);
        assert_eq!(parser.search_messages("memory").len(), 1);
        
        let counts = parser.count_by_level();
        assert_eq!(counts.get("ERROR"), Some(&1));
        assert_eq!(counts.get("INFO"), Some(&1));
        assert_eq!(counts.get("WARN"), Some(&1));
    }
}