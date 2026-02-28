use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
}

#[derive(Debug)]
pub struct LogSummary {
    total_entries: usize,
    error_count: usize,
    warning_count: usize,
    info_count: usize,
    unique_messages: HashMap<String, usize>,
}

impl LogSummary {
    pub fn new() -> Self {
        LogSummary {
            total_entries: 0,
            error_count: 0,
            warning_count: 0,
            info_count: 0,
            unique_messages: HashMap::new(),
        }
    }

    pub fn add_entry(&mut self, entry: &LogEntry) {
        self.total_entries += 1;

        match entry.level.as_str() {
            "ERROR" => self.error_count += 1,
            "WARNING" => self.warning_count += 1,
            "INFO" => self.info_count += 1,
            _ => (),
        }

        let count = self.unique_messages.entry(entry.message.clone()).or_insert(0);
        *count += 1;
    }

    pub fn display(&self) {
        println!("Log Analysis Summary:");
        println!("Total entries: {}", self.total_entries);
        println!("Errors: {}", self.error_count);
        println!("Warnings: {}", self.warning_count);
        println!("Info messages: {}", self.info_count);
        println!("\nUnique message occurrences:");
        for (message, count) in &self.unique_messages {
            println!("  {}: {}", message, count);
        }
    }
}

pub fn parse_log_file<P: AsRef<Path>>(path: P) -> Result<Vec<LogEntry>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
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
    if parts.len() == 3 {
        Some(LogEntry {
            timestamp: parts[0].to_string(),
            level: parts[1].to_string(),
            message: parts[2].to_string(),
        })
    } else {
        None
    }
}

pub fn analyze_logs<P: AsRef<Path>>(path: P) -> Result<LogSummary, Box<dyn std::error::Error>> {
    let entries = parse_log_file(path)?;
    let mut summary = LogSummary::new();

    for entry in &entries {
        summary.add_entry(entry);
    }

    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_log_line() {
        let line = "2023-10-01T12:00:00 INFO Application started";
        let entry = parse_log_line(line).unwrap();
        
        assert_eq!(entry.timestamp, "2023-10-01T12:00:00");
        assert_eq!(entry.level, "INFO");
        assert_eq!(entry.message, "Application started");
    }

    #[test]
    fn test_log_summary() {
        let mut summary = LogSummary::new();
        
        let entry1 = LogEntry {
            timestamp: "2023-10-01T12:00:00".to_string(),
            level: "INFO".to_string(),
            message: "Application started".to_string(),
        };
        
        let entry2 = LogEntry {
            timestamp: "2023-10-01T12:01:00".to_string(),
            level: "ERROR".to_string(),
            message: "Database connection failed".to_string(),
        };
        
        summary.add_entry(&entry1);
        summary.add_entry(&entry2);
        
        assert_eq!(summary.total_entries, 2);
        assert_eq!(summary.info_count, 1);
        assert_eq!(summary.error_count, 1);
        assert_eq!(summary.unique_messages.len(), 2);
    }

    #[test]
    fn test_analyze_logs() -> Result<(), Box<dyn std::error::Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "2023-10-01T12:00:00 INFO Application started")?;
        writeln!(temp_file, "2023-10-01T12:01:00 ERROR Database connection failed")?;
        writeln!(temp_file, "2023-10-01T12:02:00 WARNING High memory usage")?;
        
        let summary = analyze_logs(temp_file.path())?;
        
        assert_eq!(summary.total_entries, 3);
        assert_eq!(summary.info_count, 1);
        assert_eq!(summary.error_count, 1);
        assert_eq!(summary.warning_count, 1);
        
        Ok(())
    }
}