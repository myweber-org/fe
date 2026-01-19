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

struct LogAnalyzer {
    entries: Vec<LogEntry>,
    summary: HashMap<String, usize>,
}

impl LogAnalyzer {
    fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
            summary: HashMap::new(),
        }
    }

    fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<LogEntry>(&line) {
                Ok(entry) => {
                    self.entries.push(entry);
                }
                Err(e) => eprintln!("Failed to parse line: {} - Error: {}", line, e),
            }
        }

        self.generate_summary();
        Ok(())
    }

    fn generate_summary(&mut self) {
        self.summary.clear();
        for entry in &self.entries {
            *self.summary.entry(entry.level.clone()).or_insert(0) += 1;
            *self.summary.entry(format!("service:{}", entry.service)).or_insert(0) += 1;
        }
    }

    fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level.eq_ignore_ascii_case(level))
            .collect()
    }

    fn filter_by_service(&self, service: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.service.eq_ignore_ascii_case(service))
            .collect()
    }

    fn get_summary(&self) -> &HashMap<String, usize> {
        &self.summary
    }

    fn export_filtered<P: AsRef<Path>>(&self, path: P, entries: Vec<&LogEntry>) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::create(path)?;
        for entry in entries {
            let json = serde_json::to_string(entry)?;
            writeln!(&mut file, "{}", json)?;
        }
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut analyzer = LogAnalyzer::new();
    
    analyzer.load_from_file("logs.jsonl")?;
    
    println!("Total entries loaded: {}", analyzer.entries.len());
    
    let error_logs = analyzer.filter_by_level("ERROR");
    println!("Error logs found: {}", error_logs.len());
    
    for (key, count) in analyzer.get_summary() {
        println!("{}: {}", key, count);
    }
    
    analyzer.export_filtered("errors.jsonl", error_logs)?;
    println!("Error logs exported to errors.jsonl");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_log_parsing() {
        let mut analyzer = LogAnalyzer::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        
        let log_data = r#"{"timestamp":"2024-01-15T10:30:00Z","level":"INFO","service":"api","message":"Request processed"}
{"timestamp":"2024-01-15T10:31:00Z","level":"ERROR","service":"database","message":"Connection failed"}
{"timestamp":"2024-01-15T10:32:00Z","level":"WARN","service":"api","message":"High latency detected"}"#;
        
        write!(temp_file, "{}", log_data).unwrap();
        
        analyzer.load_from_file(temp_file.path()).unwrap();
        assert_eq!(analyzer.entries.len(), 3);
        assert_eq!(analyzer.filter_by_level("ERROR").len(), 1);
        assert_eq!(analyzer.filter_by_service("api").len(), 2);
    }
}