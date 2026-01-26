use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, PartialEq)]
enum LogSeverity {
    Error,
    Warning,
    Info,
    Debug,
}

impl LogSeverity {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "error" => Some(LogSeverity::Error),
            "warning" => Some(LogSeverity::Warning),
            "info" => Some(LogSeverity::Info),
            "debug" => Some(LogSeverity::Debug),
            _ => None,
        }
    }
}

pub fn filter_logs_by_severity(file_path: &str, severity: LogSeverity) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut filtered_logs = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if let Ok(json_value) = serde_json::from_str::<Value>(&line) {
            if let Some(level) = json_value.get("level").and_then(|v| v.as_str()) {
                if let Some(log_severity) = LogSeverity::from_str(level) {
                    if log_severity == severity {
                        filtered_logs.push(line);
                    }
                }
            }
        }
    }

    Ok(filtered_logs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_filter_error_logs() {
        let log_data = r#"{"timestamp": "2023-10-01T12:00:00Z", "level": "error", "message": "Something went wrong"}
{"timestamp": "2023-10-01T12:01:00Z", "level": "info", "message": "System started"}
{"timestamp": "2023-10-01T12:02:00Z", "level": "error", "message": "Another error occurred"}"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", log_data).unwrap();

        let result = filter_logs_by_severity(temp_file.path().to_str().unwrap(), LogSeverity::Error).unwrap();
        assert_eq!(result.len(), 2);
        assert!(result[0].contains("Something went wrong"));
        assert!(result[1].contains("Another error occurred"));
    }

    #[test]
    fn test_severity_parsing() {
        assert_eq!(LogSeverity::from_str("error"), Some(LogSeverity::Error));
        assert_eq!(LogSeverity::from_str("ERROR"), Some(LogSeverity::Error));
        assert_eq!(LogSeverity::from_str("warning"), Some(LogSeverity::Warning));
        assert_eq!(LogSeverity::from_str("info"), Some(LogSeverity::Info));
        assert_eq!(LogSeverity::from_str("debug"), Some(LogSeverity::Debug));
        assert_eq!(LogSeverity::from_str("unknown"), None);
    }
}