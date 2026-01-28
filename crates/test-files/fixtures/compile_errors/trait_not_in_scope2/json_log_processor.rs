use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    service: String,
    message: String,
    metadata: HashMap<String, String>,
}

#[derive(Debug)]
struct LogSummary {
    total_entries: usize,
    error_count: usize,
    warning_count: usize,
    services: HashMap<String, usize>,
}

impl LogEntry {
    fn from_json(line: &str) -> Option<Self> {
        serde_json::from_str(line).ok()
    }
    
    fn is_error(&self) -> bool {
        self.level.to_lowercase() == "error"
    }
    
    fn is_warning(&self) -> bool {
        self.level.to_lowercase() == "warning"
    }
}

struct LogProcessor {
    filters: Vec<Box<dyn Fn(&LogEntry) -> bool>>,
}

impl LogProcessor {
    fn new() -> Self {
        LogProcessor { filters: Vec::new() }
    }
    
    fn add_filter<F>(&mut self, filter: F)
    where
        F: Fn(&LogEntry) -> bool + 'static,
    {
        self.filters.push(Box::new(filter));
    }
    
    fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<LogSummary, std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut summary = LogSummary {
            total_entries: 0,
            error_count: 0,
            warning_count: 0,
            services: HashMap::new(),
        };
        
        for line in reader.lines() {
            let line = line?;
            if let Some(entry) = LogEntry::from_json(&line) {
                if self.filters.iter().all(|f| f(&entry)) {
                    summary.total_entries += 1;
                    
                    if entry.is_error() {
                        summary.error_count += 1;
                    } else if entry.is_warning() {
                        summary.warning_count += 1;
                    }
                    
                    *summary.services.entry(entry.service.clone()).or_insert(0) += 1;
                }
            }
        }
        
        Ok(summary)
    }
}

fn main() {
    let mut processor = LogProcessor::new();
    
    processor.add_filter(|entry| entry.level != "debug");
    
    processor.add_filter(|entry| entry.service == "api" || entry.service == "database");
    
    match processor.process_file("logs/app.log") {
        Ok(summary) => {
            println!("Log Analysis Summary:");
            println!("Total entries processed: {}", summary.total_entries);
            println!("Errors: {}", summary.error_count);
            println!("Warnings: {}", summary.warning_count);
            println!("\nEntries per service:");
            for (service, count) in summary.services {
                println!("  {}: {}", service, count);
            }
        }
        Err(e) => eprintln!("Failed to process log file: {}", e),
    }
}