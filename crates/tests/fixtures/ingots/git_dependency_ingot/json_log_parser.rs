
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    service: String,
    message: String,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

struct LogParser {
    entries: Vec<LogEntry>,
    stats: HashMap<String, usize>,
}

impl LogParser {
    fn new() -> Self {
        LogParser {
            entries: Vec::new(),
            stats: HashMap::new(),
        }
    }

    fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = serde_json::from_str::<LogEntry>(&line) {
                self.entries.push(entry);
            }
        }
        Ok(())
    }

    fn analyze(&mut self) {
        self.stats.clear();
        for entry in &self.entries {
            *self.stats.entry(entry.level.clone()).or_insert(0) += 1;
            *self.stats.entry(entry.service.clone()).or_insert(0) += 1;
        }
    }

    fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    fn filter_by_service(&self, service: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.service == service)
            .collect()
    }

    fn get_summary(&self) -> HashMap<String, usize> {
        self.stats.clone()
    }

    fn export_filtered<P: AsRef<Path>>(&self, path: P, entries: Vec<&LogEntry>) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::create(path)?;
        let mut writer = std::io::BufWriter::new(file);

        for entry in entries {
            let json = serde_json::to_string(entry)?;
            writeln!(writer, "{}", json)?;
        }
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = LogParser::new();
    
    parser.load_from_file("logs/app.log")?;
    parser.analyze();
    
    println!("Log Analysis Summary:");
    for (key, value) in parser.get_summary() {
        println!("{}: {}", key, value);
    }
    
    let errors = parser.filter_by_level("ERROR");
    println!("\nFound {} ERROR entries", errors.len());
    
    parser.export_filtered("logs/errors.json", errors)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_log_parsing() {
        let mut parser = LogParser::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        
        let log_data = r#"{"timestamp":"2023-10-01T12:00:00Z","level":"INFO","service":"api","message":"Request received"}
{"timestamp":"2023-10-01T12:00:01Z","level":"ERROR","service":"db","message":"Connection failed"}"#;
        
        writeln!(temp_file, "{}", log_data).unwrap();
        
        parser.load_from_file(temp_file.path()).unwrap();
        parser.analyze();
        
        assert_eq!(parser.entries.len(), 2);
        assert_eq!(parser.get_summary().get("INFO"), Some(&1));
        assert_eq!(parser.get_summary().get("ERROR"), Some(&1));
    }
}