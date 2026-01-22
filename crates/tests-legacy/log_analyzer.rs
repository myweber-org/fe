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
        let mut ip_counter = HashMap::new();
        let mut endpoint_counter = HashMap::new();
        
        for line in reader.lines() {
            let line = line.map_err(|e| e.to_string())?;
            self.process_line(&line, &mut summary, &mut ip_counter, &mut endpoint_counter);
        }
        
        summary.most_frequent_ip = Self::find_most_frequent(&ip_counter);
        summary.most_accessed_endpoint = Self::find_most_frequent(&endpoint_counter);
        
        Ok(summary)
    }
    
    fn process_line(&self, line: &str, summary: &mut LogSummary, 
                   ip_counter: &mut HashMap<String, u32>,
                   endpoint_counter: &mut HashMap<String, u32>) {
        summary.total_lines += 1;
        
        if self.error_pattern.is_match(line) {
            summary.error_count += 1;
        } else if self.warning_pattern.is_match(line) {
            summary.warning_count += 1;
        } else if self.info_pattern.is_match(line) {
            summary.info_count += 1;
        }
        
        if let Some(ip) = Self::extract_ip(line) {
            *ip_counter.entry(ip).or_insert(0) += 1;
        }
        
        if let Some(endpoint) = Self::extract_endpoint(line) {
            *endpoint_counter.entry(endpoint).or_insert(0) += 1;
        }
    }
    
    fn extract_ip(line: &str) -> Option<String> {
        let ip_pattern = Regex::new(r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b").unwrap();
        ip_pattern.find(line).map(|m| m.as_str().to_string())
    }
    
    fn extract_endpoint(line: &str) -> Option<String> {
        let endpoint_pattern = Regex::new(r"GET|POST|PUT|DELETE\s+(\S+)").unwrap();
        endpoint_pattern.captures(line)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }
    
    fn find_most_frequent<T: Eq + std::hash::Hash + Clone>(counter: &HashMap<T, u32>) -> Option<(T, u32)> {
        counter.iter()
            .max_by_key(|&(_, count)| count)
            .map(|(key, &count)| (key.clone(), count))
    }
}

pub struct LogSummary {
    pub total_lines: u32,
    pub error_count: u32,
    pub warning_count: u32,
    pub info_count: u32,
    pub most_frequent_ip: Option<(String, u32)>,
    pub most_accessed_endpoint: Option<(String, u32)>,
}

impl LogSummary {
    fn new() -> Self {
        LogSummary {
            total_lines: 0,
            error_count: 0,
            warning_count: 0,
            info_count: 0,
            most_frequent_ip: None,
            most_accessed_endpoint: None,
        }
    }
    
    pub fn print_summary(&self) {
        println!("Log Analysis Summary:");
        println!("Total lines: {}", self.total_lines);
        println!("Errors: {}", self.error_count);
        println!("Warnings: {}", self.warning_count);
        println!("Info messages: {}", self.info_count);
        
        if let Some((ip, count)) = &self.most_frequent_ip {
            println!("Most frequent IP: {} ({} requests)", ip, count);
        }
        
        if let Some((endpoint, count)) = &self.most_accessed_endpoint {
            println!("Most accessed endpoint: {} ({} requests)", endpoint, count);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_log_analyzer() {
        let analyzer = LogAnalyzer::new();
        let test_log = "INFO: User login from 192.168.1.100\n\
                       ERROR: Database connection failed\n\
                       WARN: High memory usage detected\n\
                       INFO: GET /api/users status=200\n\
                       INFO: POST /api/login from 192.168.1.100";
        
        let mut summary = LogSummary::new();
        let mut ip_counter = HashMap::new();
        let mut endpoint_counter = HashMap::new();
        
        for line in test_log.lines() {
            analyzer.process_line(line, &mut summary, &mut ip_counter, &mut endpoint_counter);
        }
        
        assert_eq!(summary.total_lines, 5);
        assert_eq!(summary.error_count, 1);
        assert_eq!(summary.warning_count, 1);
        assert_eq!(summary.info_count, 3);
    }
}