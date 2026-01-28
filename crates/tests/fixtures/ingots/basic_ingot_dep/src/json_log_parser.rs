use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum LogLevel {
    INFO,
    WARN,
    ERROR,
    DEBUG,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub service: String,
    pub message: String,
    pub metadata: Option<serde_json::Value>,
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

    pub fn parse_logs(&self) -> Result<Vec<LogEntry>, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut logs = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let log_entry: LogEntry = serde_json::from_str(&line)?;
            logs.push(log_entry);
        }

        Ok(logs)
    }

    pub fn filter_by_level(&self, level: LogLevel) -> Result<Vec<LogEntry>, Box<dyn Error>> {
        let logs = self.parse_logs()?;
        let filtered: Vec<LogEntry> = logs
            .into_iter()
            .filter(|log| log.level == level)
            .collect();

        Ok(filtered)
    }

    pub fn count_logs_by_service(&self) -> Result<std::collections::HashMap<String, usize>, Box<dyn Error>> {
        let logs = self.parse_logs()?;
        let mut counts = std::collections::HashMap::new();

        for log in logs {
            *counts.entry(log.service).or_insert(0) += 1;
        }

        Ok(counts)
    }
}

pub fn write_filtered_logs(output_path: &str, logs: &[LogEntry]) -> Result<(), Box<dyn Error>> {
    let file = File::create(output_path)?;
    let mut writer = std::io::BufWriter::new(file);

    for log in logs {
        let json_line = serde_json::to_string(log)?;
        writeln!(writer, "{}", json_line)?;
    }

    Ok(())
}