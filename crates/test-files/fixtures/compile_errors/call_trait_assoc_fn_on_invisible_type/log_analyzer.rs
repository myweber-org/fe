
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

pub struct LogAnalyzer {
    error_pattern: Regex,
    warning_pattern: Regex,
    info_pattern: Regex,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            error_pattern: Regex::new(r"ERROR|error|Error").unwrap(),
            warning_pattern: Regex::new(r"WARN|warn|Warn|warning|Warning").unwrap(),
            info_pattern: Regex::new(r"INFO|info|Info").unwrap(),
        }
    }

    pub fn analyze_file(&self, file_path: &str) -> Result<LogSummary, String> {
        let file = File::open(file_path)
            .map_err(|e| format!("Failed to open file: {}", e))?;
        
        let reader = BufReader::new(file);
        let mut summary = LogSummary::new();
        
        for (line_num, line_result) in reader.lines().enumerate() {
            let line = line_result
                .map_err(|e| format!("Failed to read line {}: {}", line_num + 1, e))?;
            
            self.analyze_line(&line, &mut summary);
        }
        
        Ok(summary)
    }
    
    fn analyze_line(&self, line: &str, summary: &mut LogSummary) {
        if self.error_pattern.is_match(line) {
            summary.error_count += 1;
            summary.add_error_line(line);
        } else if self.warning_pattern.is_match(line) {
            summary.warning_count += 1;
        } else if self.info_pattern.is_match(line) {
            summary.info_count += 1;
        }
        
        summary.total_lines += 1;
    }
}

pub struct LogSummary {
    pub total_lines: usize,
    pub error_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub error_lines: Vec<String>,
}

impl LogSummary {
    fn new() -> Self {
        LogSummary {
            total_lines: 0,
            error_count: 0,
            warning_count: 0,
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
        println!("Warnings: {}", self.warning_count);
        println!("Info messages: {}", self.info_count);
        
        if !self.error_lines.is_empty() {
            println!("\nRecent error lines:");
            for line in &self.error_lines {
                println!("  {}", line);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_log_analyzer() {
        let analyzer = LogAnalyzer::new();
        let test_log = "INFO: Application started\nERROR: Something went wrong\nWARN: This is a warning\n";
        
        let mut summary = LogSummary::new();
        for line in test_log.lines() {
            analyzer.analyze_line(line, &mut summary);
        }
        
        assert_eq!(summary.total_lines, 3);
        assert_eq!(summary.error_count, 1);
        assert_eq!(summary.warning_count, 1);
        assert_eq!(summary.info_count, 1);
    }
}