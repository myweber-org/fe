use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

pub fn parse_log_file<P: AsRef<Path>>(path: P) -> io::Result<Vec<LogEntry>> {
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);
    let mut entries = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if let Some(entry) = parse_log_line(&line) {
            entries.push(entry);
        }
    }

    Ok(entries)
}

fn parse_log_line(line: &str) -> Option<LogEntry> {
    let parts: Vec<&str> = line.splitn(3, ' ').collect();
    if parts.len() < 3 {
        return None;
    }

    let timestamp = parts[0].to_string();
    let level = parts[1].to_string();
    let message = parts[2].to_string();

    Some(LogEntry {
        timestamp,
        level,
        message,
    })
}

pub fn filter_by_level(entries: &[LogEntry], level: &str) -> Vec<LogEntry> {
    entries
        .iter()
        .filter(|entry| entry.level == level)
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_log_line() {
        let line = "2023-10-05T14:30:00Z ERROR Database connection failed";
        let entry = parse_log_line(line).unwrap();
        
        assert_eq!(entry.timestamp, "2023-10-05T14:30:00Z");
        assert_eq!(entry.level, "ERROR");
        assert_eq!(entry.message, "Database connection failed");
    }

    #[test]
    fn test_filter_by_level() {
        let entries = vec![
            LogEntry {
                timestamp: "2023-10-05T14:30:00Z".to_string(),
                level: "ERROR".to_string(),
                message: "Database connection failed".to_string(),
            },
            LogEntry {
                timestamp: "2023-10-05T14:31:00Z".to_string(),
                level: "INFO".to_string(),
                message: "Server started".to_string(),
            },
            LogEntry {
                timestamp: "2023-10-05T14:32:00Z".to_string(),
                level: "ERROR".to_string(),
                message: "Memory allocation failed".to_string(),
            },
        ];

        let errors = filter_by_level(&entries, "ERROR");
        assert_eq!(errors.len(), 2);
        assert!(errors.iter().all(|e| e.level == "ERROR"));
    }
}