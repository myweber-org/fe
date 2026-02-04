use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize, Serialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    service: String,
}

struct LogParser {
    file_path: String,
}

impl LogParser {
    fn new(file_path: &str) -> Self {
        LogParser {
            file_path: file_path.to_string(),
        }
    }

    fn parse_logs(&self) -> Result<Vec<LogEntry>, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut logs = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let log_entry: LogEntry = serde_json::from_str(&line)?;
            logs.push(log_entry);
        }

        Ok(logs)
    }

    fn filter_by_level(&self, level: &str) -> Result<Vec<LogEntry>, Box<dyn Error>> {
        let logs = self.parse_logs()?;
        let filtered: Vec<LogEntry> = logs
            .into_iter()
            .filter(|log| log.level.to_lowercase() == level.to_lowercase())
            .collect();
        Ok(filtered)
    }

    fn filter_by_time_range(
        &self,
        start_time: &str,
        end_time: &str,
    ) -> Result<Vec<LogEntry>, Box<dyn Error>> {
        let logs = self.parse_logs()?;
        let start: DateTime<Utc> = start_time.parse()?;
        let end: DateTime<Utc> = end_time.parse()?;

        let filtered: Vec<LogEntry> = logs
            .into_iter()
            .filter(|log| {
                if let Ok(log_time) = log.timestamp.parse::<DateTime<Utc>>() {
                    log_time >= start && log_time <= end
                } else {
                    false
                }
            })
            .collect();

        Ok(filtered)
    }

    fn count_logs_by_service(&self) -> Result<std::collections::HashMap<String, usize>, Box<dyn Error>> {
        let logs = self.parse_logs()?;
        let mut service_counts = std::collections::HashMap::new();

        for log in logs {
            *service_counts.entry(log.service).or_insert(0) += 1;
        }

        Ok(service_counts)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let parser = LogParser::new("logs.jsonl");
    
    println!("Parsing all logs...");
    let all_logs = parser.parse_logs()?;
    println!("Total logs: {}", all_logs.len());

    println!("\nFiltering ERROR logs...");
    let error_logs = parser.filter_by_level("error")?;
    println!("Error logs count: {}", error_logs.len());

    println!("\nLog counts by service:");
    let service_counts = parser.count_logs_by_service()?;
    for (service, count) in service_counts {
        println!("{}: {}", service, count);
    }

    Ok(())
}