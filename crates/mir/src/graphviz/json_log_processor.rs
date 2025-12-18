
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    service: String,
    message: String,
    metadata: Option<serde_json::Value>,
}

#[derive(Debug)]
struct LogProcessor {
    entries: Vec<LogEntry>,
    error_count: usize,
}

impl LogProcessor {
    fn new() -> Self {
        LogProcessor {
            entries: Vec::new(),
            error_count: 0,
        }
    }

    fn process_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            match self.parse_log_line(&line) {
                Ok(entry) => self.entries.push(entry),
                Err(e) => {
                    eprintln!("Error parsing line {}: {}", line_num + 1, e);
                    self.error_count += 1;
                }
            }
        }

        Ok(())
    }

    fn parse_log_line(&self, line: &str) -> Result<LogEntry, Box<dyn Error>> {
        let parsed: serde_json::Value = serde_json::from_str(line)?;
        
        let timestamp = parsed["timestamp"]
            .as_str()
            .ok_or("Missing timestamp field")?
            .to_string();
            
        let level = parsed["level"]
            .as_str()
            .ok_or("Missing level field")?
            .to_string();
            
        let service = parsed["service"]
            .as_str()
            .ok_or("Missing service field")?
            .to_string();
            
        let message = parsed["message"]
            .as_str()
            .ok_or("Missing message field")?
            .to_string();
            
        let metadata = parsed.get("metadata").cloned();

        Ok(LogEntry {
            timestamp,
            level,
            service,
            message,
            metadata,
        })
    }

    fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.eq_ignore_ascii_case(level))
            .collect()
    }

    fn get_stats(&self) -> (usize, usize) {
        (self.entries.len(), self.error_count)
    }

    fn export_to_json(&self) -> Result<String, Box<dyn Error>> {
        let output = serde_json::to_string_pretty(&self.entries)?;
        Ok(output)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut processor = LogProcessor::new();
    
    match processor.process_file("logs/app.log") {
        Ok(_) => {
            let (processed, errors) = processor.get_stats();
            println!("Processed {} entries, {} errors", processed, errors);
            
            let error_logs = processor.filter_by_level("error");
            println!("Found {} error logs", error_logs.len());
            
            if !error_logs.is_empty() {
                for log in error_logs.iter().take(3) {
                    println!("Error: {} - {}", log.timestamp, log.message);
                }
            }
            
            let json_output = processor.export_to_json()?;
            std::fs::write("logs/processed.json", json_output)?;
        }
        Err(e) => eprintln!("Failed to process log file: {}", e),
    }
    
    Ok(())
}