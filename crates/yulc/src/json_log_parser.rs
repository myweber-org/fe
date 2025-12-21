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
}