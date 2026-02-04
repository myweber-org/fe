use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum LogSeverity {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

struct LogEntry {
    timestamp: String,
    severity: LogSeverity,
    component: String,
    message: String,
}

struct LogProcessor {
    filters: HashMap<LogSeverity, bool>,
    component_filter: Option<String>,
}

impl LogProcessor {
    fn new() -> Self {
        let mut filters = HashMap::new();
        filters.insert(LogSeverity::Debug, true);
        filters.insert(LogSeverity::Info, true);
        filters.insert(LogSeverity::Warning, true);
        filters.insert(LogSeverity::Error, true);
        filters.insert(LogSeverity::Critical, true);

        LogProcessor {
            filters,
            component_filter: None,
        }
    }

    fn set_severity_filter(&mut self, severity: LogSeverity, enabled: bool) {
        self.filters.insert(severity, enabled);
    }

    fn set_component_filter(&mut self, component: Option<String>) {
        self.component_filter = component;
    }

    fn parse_line(&self, line: &str) -> Option<LogEntry> {
        let parts: Vec<&str> = line.splitn(4, '|').collect();
        if parts.len() != 4 {
            return None;
        }

        let severity = match parts[1].trim() {
            "DEBUG" => LogSeverity::Debug,
            "INFO" => LogSeverity::Info,
            "WARN" => LogSeverity::Warning,
            "ERROR" => LogSeverity::Error,
            "CRITICAL" => LogSeverity::Critical,
            _ => return None,
        };

        Some(LogEntry {
            timestamp: parts[0].trim().to_string(),
            severity,
            component: parts[2].trim().to_string(),
            message: parts[3].trim().to_string(),
        })
    }

    fn should_include(&self, entry: &LogEntry) -> bool {
        if let Some(enabled) = self.filters.get(&entry.severity) {
            if !enabled {
                return false;
            }
        }

        if let Some(ref filter) = self.component_filter {
            if !entry.component.contains(filter) {
                return false;
            }
        }

        true
    }

    fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<LogEntry>, std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Some(entry) = self.parse_line(&line) {
                if self.should_include(&entry) {
                    entries.push(entry);
                }
            }
        }

        Ok(entries)
    }

    fn generate_summary(&self, entries: &[LogEntry]) -> HashMap<LogSeverity, usize> {
        let mut summary = HashMap::new();
        for entry in entries {
            *summary.entry(entry.severity.clone()).or_insert(0) += 1;
        }
        summary
    }
}

fn main() {
    let mut processor = LogProcessor::new();
    processor.set_severity_filter(LogSeverity::Debug, false);
    processor.set_component_filter(Some("database".to_string()));

    match processor.process_file("application.log") {
        Ok(entries) => {
            println!("Found {} relevant log entries", entries.len());
            let summary = processor.generate_summary(&entries);
            for (severity, count) in summary {
                println!("{:?}: {}", severity, count);
            }
        }
        Err(e) => eprintln!("Error processing log file: {}", e),
    }
}