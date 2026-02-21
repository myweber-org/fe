use serde::Deserialize;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, PartialEq)]
enum LogLevel {
    ERROR,
    WARN,
    INFO,
    DEBUG,
    TRACE,
}

#[derive(Debug, Deserialize)]
struct LogEntry {
    timestamp: String,
    level: LogLevel,
    message: String,
    module: Option<String>,
}

struct LogParser {
    file_path: String,
    min_level: LogLevel,
}

impl LogParser {
    fn new(file_path: &str, min_level: LogLevel) -> Self {
        LogParser {
            file_path: file_path.to_string(),
            min_level,
        }
    }

    fn parse(&self) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
        let path = Path::new(&self.file_path);
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
                    if self.should_include(&entry.level) {
                        entries.push(entry);
                    }
                }
                Err(e) => eprintln!("Failed to parse line: {} - {}", line, e),
            }
        }

        Ok(entries)
    }

    fn should_include(&self, level: &LogLevel) -> bool {
        match (&self.min_level, level) {
            (LogLevel::ERROR, _) => level == &LogLevel::ERROR,
            (LogLevel::WARN, _) => matches!(level, LogLevel::ERROR | LogLevel::WARN),
            (LogLevel::INFO, _) => matches!(level, LogLevel::ERROR | LogLevel::WARN | LogLevel::INFO),
            (LogLevel::DEBUG, _) => matches!(level, LogLevel::ERROR | LogLevel::WARN | LogLevel::INFO | LogLevel::DEBUG),
            (LogLevel::TRACE, _) => true,
        }
    }

    fn count_by_level(&self) -> Result<std::collections::HashMap<LogLevel, usize>, Box<dyn std::error::Error>> {
        let entries = self.parse()?;
        let mut counts = std::collections::HashMap::new();

        for entry in entries {
            *counts.entry(entry.level).or_insert(0) += 1;
        }

        Ok(counts)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parser = LogParser::new("application.log", LogLevel::INFO);
    let counts = parser.count_by_level()?;

    for (level, count) in counts {
        println!("{:?}: {}", level, count);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_parsing() {
        let log_data = r#"{"timestamp":"2023-10-01T12:00:00Z","level":"ERROR","message":"Failed to connect","module":"network"}
{"timestamp":"2023-10-01T12:01:00Z","level":"INFO","message":"Server started","module":"server"}
{"timestamp":"2023-10-01T12:02:00Z","level":"DEBUG","message":"Processing request","module":"server"}"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", log_data).unwrap();

        let parser = LogParser::new(temp_file.path().to_str().unwrap(), LogLevel::INFO);
        let entries = parser.parse().unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].level, LogLevel::ERROR);
        assert_eq!(entries[1].level, LogLevel::INFO);
    }

    #[test]
    fn test_level_filtering() {
        let parser = LogParser::new("dummy.log", LogLevel::WARN);
        
        assert!(parser.should_include(&LogLevel::ERROR));
        assert!(parser.should_include(&LogLevel::WARN));
        assert!(!parser.should_include(&LogLevel::INFO));
        assert!(!parser.should_include(&LogLevel::DEBUG));
        assert!(!parser.should_include(&LogLevel::TRACE));
    }
}