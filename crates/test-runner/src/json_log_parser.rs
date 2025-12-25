use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use serde_json::Value;

#[derive(Debug, PartialEq)]
enum LogSeverity {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

impl LogSeverity {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "debug" => Some(LogSeverity::Debug),
            "info" => Some(LogSeverity::Info),
            "warning" => Some(LogSeverity::Warning),
            "error" => Some(LogSeverity::Error),
            "critical" => Some(LogSeverity::Critical),
            _ => None,
        }
    }
}

pub struct LogParser {
    file_path: String,
    severity_filter: Option<LogSeverity>,
}

impl LogParser {
    pub fn new(file_path: &str) -> Self {
        LogParser {
            file_path: file_path.to_string(),
            severity_filter: None,
        }
    }

    pub fn set_severity_filter(&mut self, severity: &str) -> Result<(), String> {
        match LogSeverity::from_str(severity) {
            Some(sev) => {
                self.severity_filter = Some(sev);
                Ok(())
            }
            None => Err(format!("Invalid severity level: {}", severity)),
        }
    }

    pub fn parse(&self) -> Result<Vec<HashMap<String, String>>, Box<dyn std::error::Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut logs = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let json_value: Value = serde_json::from_str(&line)?;
            let mut log_entry = HashMap::new();

            if let Some(obj) = json_value.as_object() {
                for (key, value) in obj {
                    let val_str = match value {
                        Value::String(s) => s.clone(),
                        Value::Number(n) => n.to_string(),
                        Value::Bool(b) => b.to_string(),
                        Value::Null => "null".to_string(),
                        _ => value.to_string(),
                    };
                    log_entry.insert(key.clone(), val_str);
                }

                if let Some(filter) = &self.severity_filter {
                    if let Some(severity_str) = log_entry.get("severity") {
                        if let Some(entry_severity) = LogSeverity::from_str(severity_str) {
                            if &entry_severity == filter {
                                logs.push(log_entry);
                            }
                        }
                    }
                } else {
                    logs.push(log_entry);
                }
            }
        }

        Ok(logs)
    }

    pub fn count_by_severity(&self) -> Result<HashMap<String, usize>, Box<dyn std::error::Error>> {
        let logs = self.parse()?;
        let mut counts = HashMap::new();

        for log in logs {
            if let Some(severity) = log.get("severity") {
                *counts.entry(severity.clone()).or_insert(0) += 1;
            }
        }

        Ok(counts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_logs() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        let logs = r#"
{"timestamp": "2023-10-01T12:00:00Z", "severity": "info", "message": "Application started"}
{"timestamp": "2023-10-01T12:01:00Z", "severity": "warning", "message": "High memory usage"}
{"timestamp": "2023-10-01T12:02:00Z", "severity": "error", "message": "Database connection failed"}
{"timestamp": "2023-10-01T12:03:00Z", "severity": "info", "message": "User login successful"}
{"timestamp": "2023-10-01T12:04:00Z", "severity": "error", "message": "API timeout"}
"#;
        write!(file, "{}", logs).unwrap();
        file
    }

    #[test]
    fn test_parse_all_logs() {
        let test_file = create_test_logs();
        let parser = LogParser::new(test_file.path().to_str().unwrap());
        let logs = parser.parse().unwrap();
        assert_eq!(logs.len(), 5);
    }

    #[test]
    fn test_filter_by_severity() {
        let test_file = create_test_logs();
        let mut parser = LogParser::new(test_file.path().to_str().unwrap());
        parser.set_severity_filter("error").unwrap();
        let logs = parser.parse().unwrap();
        assert_eq!(logs.len(), 2);
        for log in logs {
            assert_eq!(log.get("severity").unwrap(), "error");
        }
    }

    #[test]
    fn test_count_by_severity() {
        let test_file = create_test_logs();
        let parser = LogParser::new(test_file.path().to_str().unwrap());
        let counts = parser.count_by_severity().unwrap();
        assert_eq!(counts.get("info"), Some(&2));
        assert_eq!(counts.get("warning"), Some(&1));
        assert_eq!(counts.get("error"), Some(&2));
    }
}