use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub enum ParseError {
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

pub struct LogParser {
    file_path: String,
}

impl LogParser {
    pub fn new(file_path: &str) -> Self {
        LogParser {
            file_path: file_path.to_string(),
        }
    }

    pub fn parse(&self) -> Result<Vec<Value>, ParseError> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut logs = Vec::new();
        
        for (line_num, line) in reader.lines().enumerate() {
            let line_content = line?;
            
            if line_content.trim().is_empty() {
                continue;
            }
            
            let json_value: Value = serde_json::from_str(&line_content)
                .map_err(|e| {
                    ParseError::InvalidLogFormat(
                        format!("Line {}: {} - {}", line_num + 1, e, line_content)
                    )
                })?;
            
            logs.push(json_value);
        }
        
        Ok(logs)
    }
    
    pub fn filter_by_level(&self, level: &str) -> Result<Vec<Value>, ParseError> {
        let logs = self.parse()?;
        let filtered: Vec<Value> = logs
            .into_iter()
            .filter(|log| {
                log.get("level")
                    .and_then(|v| v.as_str())
                    .map(|lvl| lvl.eq_ignore_ascii_case(level))
                    .unwrap_or(false)
            })
            .collect();
        
        Ok(filtered)
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
        writeln!(temp_file, r#"{{"level": "INFO", "message": "System started", "timestamp": "2024-01-01T00:00:00Z"}}"#).unwrap();
        writeln!(temp_file, r#"{{"level": "ERROR", "message": "Disk full", "timestamp": "2024-01-01T00:01:00Z"}}"#).unwrap();
        
        let parser = LogParser::new(temp_file.path().to_str().unwrap());
        let result = parser.parse();
        
        assert!(result.is_ok());
        let logs = result.unwrap();
        assert_eq!(logs.len(), 2);
    }
    
    #[test]
    fn test_filter_by_level() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{"level": "INFO", "message": "Test"}}"#).unwrap();
        writeln!(temp_file, r#"{{"level": "ERROR", "message": "Error"}}"#).unwrap();
        writeln!(temp_file, r#"{{"level": "INFO", "message": "Another"}}"#).unwrap();
        
        let parser = LogParser::new(temp_file.path().to_str().unwrap());
        let errors = parser.filter_by_level("ERROR").unwrap();
        
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].get("level").unwrap().as_str().unwrap(), "ERROR");
    }
}use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Deserialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug)]
struct LogFilter {
    min_level: Option<String>,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
    keyword: Option<String>,
}

impl LogFilter {
    fn matches(&self, entry: &LogEntry) -> bool {
        if let Some(min_level) = &self.min_level {
            let levels = vec!["trace", "debug", "info", "warn", "error"];
            let entry_idx = levels.iter().position(|&l| l == entry.level.to_lowercase());
            let min_idx = levels.iter().position(|&l| l == min_level.to_lowercase());
            
            if let (Some(e_idx), Some(m_idx)) = (entry_idx, min_idx) {
                if e_idx < m_idx {
                    return false;
                }
            }
        }

        if let (Some(start), Some(end)) = (&self.start_time, &self.end_time) {
            if let Ok(entry_time) = DateTime::parse_from_rfc3339(&entry.timestamp) {
                let entry_utc = entry_time.with_timezone(&Utc);
                if entry_utc < *start || entry_utc > *end {
                    return false;
                }
            }
        }

        if let Some(keyword) = &self.keyword {
            if !entry.message.contains(keyword) {
                return false;
            }
        }

        true
    }
}

fn parse_log_file(path: &str, filter: &LogFilter) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<LogEntry>(&line) {
            Ok(entry) => {
                if filter.matches(&entry) {
                    entries.push(entry);
                }
            }
            Err(e) => eprintln!("Failed to parse line: {} - Error: {}", line, e),
        }
    }

    Ok(entries)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filter = LogFilter {
        min_level: Some("info".to_string()),
        start_time: Some(Utc::now() - chrono::Duration::hours(24)),
        end_time: Some(Utc::now()),
        keyword: Some("connection".to_string()),
    };

    let entries = parse_log_file("application.log", &filter)?;
    
    println!("Found {} matching log entries:", entries.len());
    for entry in entries.iter().take(5) {
        println!("[{}] {}: {}", entry.timestamp, entry.level, entry.message);
    }
    
    Ok(())
}