use std::fs::File;
use std::io::{BufRead, BufReader};
use chrono::{DateTime, FixedOffset};
use serde_json::Value;

#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: DateTime<FixedOffset>,
    pub level: String,
    pub message: String,
    pub raw_data: Value,
}

pub struct LogParser {
    file_path: String,
}

impl LogParser {
    pub fn new(file_path: &str) -> Self {
        LogParser {
            file_path: file_path.to_string(),
        }
    }

    pub fn parse_with_filters(
        &self,
        min_level: Option<&str>,
        start_time: Option<DateTime<FixedOffset>>,
        end_time: Option<DateTime<FixedOffset>>,
    ) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let json_value: Value = serde_json::from_str(&line)?;

            let timestamp_str = json_value["timestamp"]
                .as_str()
                .ok_or("Missing timestamp field")?;
            let timestamp = DateTime::parse_from_rfc3339(timestamp_str)?;

            if let Some(start) = start_time {
                if timestamp < start {
                    continue;
                }
            }

            if let Some(end) = end_time {
                if timestamp > end {
                    continue;
                }
            }

            let level = json_value["level"]
                .as_str()
                .ok_or("Missing level field")?
                .to_string();

            if let Some(min_lvl) = min_level {
                if !self.is_level_allowed(&level, min_lvl) {
                    continue;
                }
            }

            let message = json_value["message"]
                .as_str()
                .unwrap_or("")
                .to_string();

            entries.push(LogEntry {
                timestamp,
                level,
                message,
                raw_data: json_value,
            });
        }

        Ok(entries)
    }

    fn is_level_allowed(&self, entry_level: &str, min_level: &str) -> bool {
        let levels = ["trace", "debug", "info", "warn", "error"];
        let entry_idx = levels.iter().position(|&l| l == entry_level.to_lowercase());
        let min_idx = levels.iter().position(|&l| l == min_level.to_lowercase());

        match (entry_idx, min_idx) {
            (Some(e), Some(m)) => e >= m,
            _ => false,
        }
    }

    pub fn count_by_level(&self) -> Result<std::collections::HashMap<String, usize>, Box<dyn std::error::Error>> {
        let entries = self.parse_with_filters(None, None, None)?;
        let mut counts = std::collections::HashMap::new();

        for entry in entries {
            *counts.entry(entry.level).or_insert(0) += 1;
        }

        Ok(counts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_level_filtering() {
        let parser = LogParser::new("test_logs.json");
        let start_time = FixedOffset::east_opt(3600)
            .unwrap()
            .with_ymd_and_hms(2024, 1, 1, 0, 0, 0)
            .unwrap();
        
        let result = parser.parse_with_filters(Some("info"), Some(start_time), None);
        assert!(result.is_ok());
    }
}