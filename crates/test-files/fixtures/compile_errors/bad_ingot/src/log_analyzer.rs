use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct LogSummary {
    pub total_lines: usize,
    pub error_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub ip_addresses: HashMap<String, usize>,
    pub status_codes: HashMap<u16, usize>,
}

impl LogSummary {
    pub fn new() -> Self {
        LogSummary {
            total_lines: 0,
            error_count: 0,
            warning_count: 0,
            info_count: 0,
            ip_addresses: HashMap::new(),
            status_codes: HashMap::new(),
        }
    }

    pub fn analyze_file<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut summary = LogSummary::new();

        for line in reader.lines() {
            let line = line?;
            summary.process_line(&line);
        }

        Ok(summary)
    }

    fn process_line(&mut self, line: &str) {
        self.total_lines += 1;

        if line.contains("ERROR") {
            self.error_count += 1;
        } else if line.contains("WARNING") {
            self.warning_count += 1;
        } else if line.contains("INFO") {
            self.info_count += 1;
        }

        self.extract_ip_address(line);
        self.extract_status_code(line);
    }

    fn extract_ip_address(&mut self, line: &str) {
        let ip_pattern = r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b";
        if let Some(captures) = regex::Regex::new(ip_pattern).unwrap().find(line) {
            let ip = captures.as_str().to_string();
            *self.ip_addresses.entry(ip).or_insert(0) += 1;
        }
    }

    fn extract_status_code(&mut self, line: &str) {
        let status_pattern = r"\bHTTP/\d\.\d\"\s+(\d{3})\b";
        if let Some(captures) = regex::Regex::new(status_pattern).unwrap().captures(line) {
            if let Some(status_str) = captures.get(1) {
                if let Ok(status_code) = status_str.as_str().parse::<u16>() {
                    *self.status_codes.entry(status_code).or_insert(0) += 1;
                }
            }
        }
    }

    pub fn print_summary(&self) {
        println!("Log Analysis Summary:");
        println!("=====================");
        println!("Total lines: {}", self.total_lines);
        println!("Errors: {}", self.error_count);
        println!("Warnings: {}", self.warning_count);
        println!("Info messages: {}", self.info_count);
        println!("\nTop IP addresses:");
        for (ip, count) in self.get_top_ips(5) {
            println!("  {}: {}", ip, count);
        }
        println!("\nStatus codes:");
        for (code, count) in &self.status_codes {
            println!("  {}: {}", code, count);
        }
    }

    fn get_top_ips(&self, n: usize) -> Vec<(&String, &usize)> {
        let mut entries: Vec<_> = self.ip_addresses.iter().collect();
        entries.sort_by(|a, b| b.1.cmp(a.1));
        entries.into_iter().take(n).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_analysis() {
        let log_content = r#"192.168.1.1 - - [10/Oct/2023:13:55:36 +0000] "GET /api/users HTTP/1.1" 200 1234 INFO: User login successful
192.168.1.2 - - [10/Oct/2023:13:55:37 +0000] "POST /api/data HTTP/1.1" 404 567 ERROR: Resource not found
192.168.1.1 - - [10/Oct/2023:13:55:38 +0000] "GET /api/status HTTP/1.1" 500 789 WARNING: Service degraded"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", log_content).unwrap();

        let summary = LogSummary::analyze_file(temp_file.path()).unwrap();
        
        assert_eq!(summary.total_lines, 3);
        assert_eq!(summary.error_count, 1);
        assert_eq!(summary.warning_count, 1);
        assert_eq!(summary.info_count, 1);
        assert_eq!(summary.ip_addresses.get("192.168.1.1"), Some(&2));
        assert_eq!(summary.ip_addresses.get("192.168.1.2"), Some(&1));
        assert_eq!(summary.status_codes.get(&200), Some(&1));
        assert_eq!(summary.status_codes.get(&404), Some(&1));
        assert_eq!(summary.status_codes.get(&500), Some(&1));
    }
}