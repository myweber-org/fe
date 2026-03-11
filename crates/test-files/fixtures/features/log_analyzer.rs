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
            warning_pattern: Regex::new(r"WARN|warn|Warn|WARNING|warning|Warning").unwrap(),
            info_pattern: Regex::new(r"INFO|info|Info").unwrap(),
        }
    }

    pub fn analyze_file(&self, file_path: &str) -> Result<LogSummary, String> {
        let file = File::open(file_path).map_err(|e| e.to_string())?;
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
    
    pub fn error_rate(&self) -> f64 {
        if self.total_lines == 0 {
            0.0
        } else {
            (self.error_count as f64 / self.total_lines as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_log_analyzer() {
        let analyzer = LogAnalyzer::new();
        let test_log = "INFO: Application started\nERROR: Failed to connect\nWARN: Retrying connection\nINFO: Connection established";
        
        let mut summary = LogSummary::new();
        for line in test_log.lines() {
            analyzer.process_line(line, &mut summary);
        }
        
        assert_eq!(summary.total_lines, 4);
        assert_eq!(summary.error_count, 1);
        assert_eq!(summary.warning_count, 1);
        assert_eq!(summary.info_count, 2);
    }
}