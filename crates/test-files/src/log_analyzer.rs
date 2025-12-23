use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

pub struct LogAnalyzer {
    error_pattern: Regex,
    warn_pattern: Regex,
    info_pattern: Regex,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            error_pattern: Regex::new(r"ERROR").unwrap(),
            warn_pattern: Regex::new(r"WARN").unwrap(),
            info_pattern: Regex::new(r"INFO").unwrap(),
        }
    }

    pub fn analyze_file(&self, file_path: &str) -> Result<HashMap<String, usize>, std::io::Error> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut stats = HashMap::new();

        for line in reader.lines() {
            let line = line?;
            self.process_line(&line, &mut stats);
        }

        Ok(stats)
    }

    fn process_line(&self, line: &str, stats: &mut HashMap<String, usize>) {
        if self.error_pattern.is_match(line) {
            *stats.entry("ERROR".to_string()).or_insert(0) += 1;
        } else if self.warn_pattern.is_match(line) {
            *stats.entry("WARN".to_string()).or_insert(0) += 1;
        } else if self.info_pattern.is_match(line) {
            *stats.entry("INFO".to_string()).or_insert(0) += 1;
        }
    }

    pub fn print_summary(&self, stats: &HashMap<String, usize>) {
        println!("Log Level Summary:");
        for (level, count) in stats {
            println!("{}: {}", level, count);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_analysis() {
        let analyzer = LogAnalyzer::new();
        let test_log = "INFO: Application started\nERROR: Failed to connect\nWARN: Retrying connection\nINFO: Connection established";
        
        let mut stats = HashMap::new();
        for line in test_log.lines() {
            analyzer.process_line(line, &mut stats);
        }

        assert_eq!(stats.get("INFO"), Some(&2));
        assert_eq!(stats.get("ERROR"), Some(&1));
        assert_eq!(stats.get("WARN"), Some(&1));
    }
}