use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LogParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parse error at line {line}: {source}")]
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
            let json_value: Value = serde_json::from_str(&line_content)
                .map_err(|e| LogParseError::JsonParse {
                    line: line_num + 1,
                    source: e,
                })?;

            if !json_value.is_object() {
                continue;
            }

            results.push(json_value);
        }

        Ok(results)
    }

    pub fn extract_field(&self, field_name: &str) -> Result<Vec<String>, LogParseError> {
        let parsed = self.parse()?;
        let mut values = Vec::new();

        for (idx, obj) in parsed.iter().enumerate() {
            if let Some(value) = obj.get(field_name) {
                values.push(value.to_string());
            } else {
                return Err(LogParseError::MissingField {
                    line: idx + 1,
                    field: field_name.to_string(),
                });
            }
        }

        Ok(values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_json_logs() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{"timestamp": "2024-01-01", "level": "INFO"}}"#).unwrap();
        writeln!(temp_file, r#"{{"timestamp": "2024-01-02", "level": "ERROR"}}"#).unwrap();

        let parser = JsonLogParser::new(temp_file.path().to_str().unwrap());
        let result = parser.parse().unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0]["level"], "INFO");
    }

    #[test]
    fn test_extract_specific_field() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, r#"{{"id": "abc123", "event": "login"}}"#).unwrap();
        writeln!(temp_file, r#"{{"id": "def456", "event": "logout"}}"#).unwrap();

        let parser = JsonLogParser::new(temp_file.path().to_str().unwrap());
        let ids = parser.extract_field("id").unwrap();
        assert_eq!(ids, ["\"abc123\"", "\"def456\""]);
    }
}