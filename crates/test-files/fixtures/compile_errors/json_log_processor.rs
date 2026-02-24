use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Deserialize, Serialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    service: String,
    message: String,
    #[serde(default)]
    metadata: serde_json::Value,
}

impl LogEntry {
    fn is_error(&self) -> bool {
        self.level.to_uppercase() == "ERROR"
    }

    fn from_service(&self, service_name: &str) -> bool {
        self.service == service_name
    }
}

struct LogProcessor {
    entries: Vec<LogEntry>,
}

impl LogProcessor {
    fn new() -> Self {
        LogProcessor { entries: Vec::new() }
    }

    fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str(&line) {
                Ok(entry) => self.entries.push(entry),
                Err(e) => eprintln!("Failed to parse line: {}. Error: {}", line, e),
            }
        }

        Ok(())
    }

    fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        let target_level = level.to_uppercase();
        self.entries
            .iter()
            .filter(|entry| entry.level.to_uppercase() == target_level)
            .collect()
    }

    fn filter_by_service(&self, service_name: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.from_service(service_name))
            .collect()
    }

    fn get_error_count(&self) -> usize {
        self.entries.iter().filter(|entry| entry.is_error()).count()
    }

    fn export_errors(&self, output_path: &str) -> Result<(), Box<dyn Error>> {
        let errors: Vec<&LogEntry> = self.filter_by_level("ERROR");
        let mut output = Vec::new();

        for error in errors {
            let json = serde_json::to_string_pretty(error)?;
            output.push(json);
        }

        std::fs::write(output_path, output.join("\n"))?;
        Ok(())
    }

    fn print_summary(&self) {
        println!("Total entries: {}", self.entries.len());
        println!("Error count: {}", self.get_error_count());

        let mut service_counts = std::collections::HashMap::new();
        for entry in &self.entries {
            *service_counts.entry(&entry.service).or_insert(0) += 1;
        }

        println!("\nEntries per service:");
        for (service, count) in service_counts {
            println!("  {}: {}", service, count);
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut processor = LogProcessor::new();
    
    match processor.load_from_file("logs.jsonl") {
        Ok(_) => {
            processor.print_summary();
            
            let errors = processor.filter_by_level("ERROR");
            if !errors.is_empty() {
                println!("\nFound {} error entries:", errors.len());
                for error in errors.iter().take(5) {
                    println!("  [{}] {}: {}", error.timestamp, error.service, error.message);
                }
                
                processor.export_errors("errors.json")?;
                println!("\nExported errors to errors.json");
            }
        }
        Err(e) => eprintln!("Failed to load log file: {}", e),
    }

    Ok(())
}