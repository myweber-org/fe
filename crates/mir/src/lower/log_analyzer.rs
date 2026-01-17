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

    pub fn analyze_file(&self, path: &str) -> Result<LogSummary, String> {
        let file = File::open(path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        
        let mut summary = LogSummary::new();
        
        for line in reader.lines() {
            let line = line.map_err(|e| e.to_string())?;
            self.process_line(&line, &mut summary);
        }
        
        Ok(summary)
    }

    fn process_line(&self, line: &str, summary: &mut LogSummary) {
        if self.error_pattern.is_match(line) {
            summary.error_count += 1;
            summary.log_levels.entry("ERROR".to_string())
                .and_modify(|count| *count += 1)
                .or_insert(1);
        } else if self.warn_pattern.is_match(line) {
            summary.warn_count += 1;
            summary.log_levels.entry("WARN".to_string())
                .and_modify(|count| *count += 1)
                .or_insert(1);
        } else if self.info_pattern.is_match(line) {
            summary.info_count += 1;
            summary.log_levels.entry("INFO".to_string())
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
        
        summary.total_lines += 1;
    }
}

pub struct LogSummary {
    pub total_lines: usize,
    pub error_count: usize,
    pub warn_count: usize,
    pub info_count: usize,
    pub log_levels: HashMap<String, usize>,
}

impl LogSummary {
    pub fn new() -> Self {
        LogSummary {
            total_lines: 0,
            error_count: 0,
            warn_count: 0,
            info_count: 0,
            log_levels: HashMap::new(),
        }
    }

    pub fn error_rate(&self) -> f64 {
        if self.total_lines == 0 {
            0.0
        } else {
            (self.error_count as f64 / self.total_lines as f64) * 100.0
        }
    }

    pub fn print_summary(&self) {
        println!("Log Analysis Summary:");
        println!("Total lines: {}", self.total_lines);
        println!("Errors: {}", self.error_count);
        println!("Warnings: {}", self.warn_count);
        println!("Info messages: {}", self.info_count);
        println!("Error rate: {:.2}%", self.error_rate());
        
        if !self.log_levels.is_empty() {
            println!("\nLog level distribution:");
            for (level, count) in &self.log_levels {
                println!("  {}: {}", level, count);
            }
        }
    }
}