use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LogParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parsing error at line {line}: {source}")]
    JsonParse {
        line: usize,
        source: serde_json::Error,
    },
    #[error("Missing required field '{field}' at line {line}")]
    MissingField { line: usize, field: String },
}

pub struct JsonLogParser {
    file_path: String,
}

impl JsonLogParser {
    pub fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
        }
    }

    pub fn parse(&self) -> Result<Vec<Value>, LogParseError> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut results = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line_content = line?;
            let line_number = line_num + 1;

            if line_content.trim().is_empty() {
                continue;
            }

            let json_value: Value = serde_json::from_str(&line_content)
                .map_err(|e| LogParseError::JsonParse {
                    line: line_number,
                    source: e,
                })?;

            if !json_value.is_object() {
                continue;
            }

            if let Some(obj) = json_value.as_object() {
                if !obj.contains_key("timestamp") {
                    return Err(LogParseError::MissingField {
                        line: line_number,
                        field: "timestamp".to_string(),
                    });
                }
            }

            results.push(json_value);
        }

        Ok(results)
    }

    pub fn filter_by_level(&self, level: &str) -> Result<Vec<Value>, LogParseError> {
        let all_logs = self.parse()?;
        let filtered: Vec<Value> = all_logs
            .into_iter()
            .filter(|log| {
                log.get("level")
                    .and_then(|v| v.as_str())
                    .map(|l| l.eq_ignore_ascii_case(level))
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
        writeln!(temp_file, r#"{{"timestamp": "2024-01-01T00:00:00Z", "level": "INFO", "message": "Test"}}"#).unwrap();
        writeln!(temp_file, r#"{{"timestamp": "2024-01-01T00:00:01Z", "level": "ERROR", "message": "Error"}}"#).unwrap();

        let parser = JsonLogParser::new(temp_file.path().to_str().unwrap());
        let result = parser.parse().unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_filter_by_level() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{"timestamp": "2024-01-01T00:00:00Z", "level": "INFO", "message": "Test"}}"#).unwrap();
        writeln!(temp_file, r#"{{"timestamp": "2024-01-01T00:00:01Z", "level": "ERROR", "message": "Error"}}"#).unwrap();

        let parser = JsonLogParser::new(temp_file.path().to_str().unwrap());
        let errors = parser.filter_by_level("ERROR").unwrap();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].get("level").unwrap().as_str().unwrap(), "ERROR");
    }
}