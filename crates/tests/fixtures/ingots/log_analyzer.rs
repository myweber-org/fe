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

    pub fn analyze_file(&self, path: &str) -> Result<LogSummary, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut summary = LogSummary::new();

        for line in reader.lines() {
            let line = line?;
            self.process_line(&line, &mut summary);
        }

        Ok(summary)
    }

    fn process_line(&self, line: &str, summary: &mut LogSummary) {
        if self.error_pattern.is_match(line) {
            summary.error_count += 1;
            summary.add_error_line(line);
        } else if self.warn_pattern.is_match(line) {
            summary.warn_count += 1;
        } else if self.info_pattern.is_match(line) {
            summary.info_count += 1;
        }
        summary.total_lines += 1;
    }
}

pub struct LogSummary {
    pub total_lines: usize,
    pub error_count: usize,
    pub warn_count: usize,
    pub info_count: usize,
    pub error_lines: Vec<String>,
}

impl LogSummary {
    fn new() -> Self {
        LogSummary {
            total_lines: 0,
            error_count: 0,
            warn_count: 0,
            info_count: 0,
            error_lines: Vec::new(),
        }
    }

    fn add_error_line(&mut self, line: &str) {
        if self.error_lines.len() < 10 {
            self.error_lines.push(line.to_string());
        }
    }

    pub fn print_summary(&self) {
        println!("Log Analysis Summary:");
        println!("Total lines: {}", self.total_lines);
        println!("Errors: {}", self.error_count);
        println!("Warnings: {}", self.warn_count);
        println!("Info messages: {}", self.info_count);
        
        if !self.error_lines.is_empty() {
            println!("\nRecent errors:");
            for error in &self.error_lines {
                println!("  {}", error);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_counts() {
        let analyzer = LogAnalyzer::new();
        let test_log = "INFO: Application started\nERROR: Database connection failed\nWARN: High memory usage\nINFO: Request processed";
        
        let mut summary = LogSummary::new();
        for line in test_log.lines() {
            analyzer.process_line(line, &mut summary);
        }

        assert_eq!(summary.total_lines, 4);
        assert_eq!(summary.error_count, 1);
        assert_eq!(summary.warn_count, 1);
        assert_eq!(summary.info_count, 2);
    }
}