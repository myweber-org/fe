use serde::Deserialize;
use std::fs::File;
use std::io::{BufRead, BufReader};
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    #[serde(flatten)]
    extra: serde_json::Value,
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Invalid log format")]
    InvalidFormat,
}

pub struct LogParser {
    file_path: String,
}

impl LogParser {
    pub fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
        }
    }

    pub fn parse(&self) -> Result<Vec<LogEntry>, ParseError> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            match serde_json::from_str::<LogEntry>(&line) {
                Ok(entry) => entries.push(entry),
                Err(e) => {
                    eprintln!("Warning: Failed to parse line {}: {}", line_num + 1, e);
                    return Err(ParseError::InvalidFormat);
                }
            }
        }

        Ok(entries)
    }

    pub fn filter_by_level(&self, level: &str) -> Result<Vec<LogEntry>, ParseError> {
        let entries = self.parse()?;
        Ok(entries
            .into_iter()
            .filter(|entry| entry.level.to_lowercase() == level.to_lowercase())
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_logs() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            r#"{{"timestamp":"2024-01-01T12:00:00Z","level":"INFO","message":"System started"}}"#
        )
        .unwrap();
        writeln!(
            temp_file,
            r#"{{"timestamp":"2024-01-01T12:01:00Z","level":"ERROR","message":"Disk full","disk_usage":95}}"#
        )
        .unwrap();

        let parser = LogParser::new(temp_file.path().to_str().unwrap());
        let result = parser.parse();
        assert!(result.is_ok());
        let entries = result.unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].level, "INFO");
        assert_eq!(entries[1].level, "ERROR");
    }

    #[test]
    fn test_filter_by_level() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            r#"{{"timestamp":"2024-01-01T12:00:00Z","level":"INFO","message":"Test"}}"#
        )
        .unwrap();
        writeln!(
            temp_file,
            r#"{{"timestamp":"2024-01-01T12:01:00Z","level":"ERROR","message":"Error"}}"#
        )
        .unwrap();

        let parser = LogParser::new(temp_file.path().to_str().unwrap());
        let errors = parser.filter_by_level("ERROR").unwrap();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "Error");
    }
}