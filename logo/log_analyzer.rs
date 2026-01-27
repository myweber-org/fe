use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

#[derive(Debug)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
}

pub struct LogAnalyzer {
    entries: Vec<LogEntry>,
    level_counts: HashMap<String, usize>,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
            level_counts: HashMap::new(),
        }
    }

    pub fn parse_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let log_pattern = Regex::new(r"(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}) \[(\w+)\] (.+)")?;

        for line in reader.lines() {
            let line = line?;
            if let Some(captures) = log_pattern.captures(&line) {
                let timestamp = captures[1].to_string();
                let level = captures[2].to_string();
                let message = captures[3].to_string();

                let entry = LogEntry {
                    timestamp,
                    level: level.clone(),
                    message,
                };

                *self.level_counts.entry(level).or_insert(0) += 1;
                self.entries.push(entry);
            }
        }

        Ok(())
    }

    pub fn get_level_summary(&self) -> &HashMap<String, usize> {
        &self.level_counts
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn total_entries(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            "2023-10-05 14:30:00 [INFO] Application started"
        ).unwrap();
        writeln!(
            temp_file,
            "2023-10-05 14:31:00 [ERROR] Failed to connect to database"
        ).unwrap();

        let mut analyzer = LogAnalyzer::new();
        analyzer.parse_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(analyzer.total_entries(), 2);
        assert_eq!(analyzer.get_level_summary().get("INFO"), Some(&1));
        assert_eq!(analyzer.get_level_summary().get("ERROR"), Some(&1));
    }
}use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

pub struct LogAnalyzer {
    error_patterns: HashMap<String, Regex>,
    warning_patterns: HashMap<String, Regex>,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        let mut error_patterns = HashMap::new();
        let mut warning_patterns = HashMap::new();

        error_patterns.insert(
            "connection_error".to_string(),
            Regex::new(r"connection.*failed|timeout|refused").unwrap(),
        );
        error_patterns.insert(
            "authentication_error".to_string(),
            Regex::new(r"auth.*failed|invalid.*credential").unwrap(),
        );

        warning_patterns.insert(
            "deprecation_warning".to_string(),
            Regex::new(r"deprecated|will.*remove").unwrap(),
        );
        warning_patterns.insert(
            "resource_warning".to_string(),
            Regex::new(r"low.*memory|high.*cpu").unwrap(),
        );

        LogAnalyzer {
            error_patterns,
            warning_patterns,
        }
    }

    pub fn analyze_log_file(&self, file_path: &str) -> Result<LogSummary, String> {
        let file = File::open(file_path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);

        let mut summary = LogSummary::new();
        let mut line_count = 0;

        for line_result in reader.lines() {
            let line = line_result.map_err(|e| e.to_string())?;
            line_count += 1;

            self.analyze_line(&line, &mut summary);
        }

        summary.total_lines = line_count;
        Ok(summary)
    }

    fn analyze_line(&self, line: &str, summary: &mut LogSummary) {
        for (error_type, pattern) in &self.error_patterns {
            if pattern.is_match(line) {
                summary.error_counts
                    .entry(error_type.clone())
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
                summary.total_errors += 1;
            }
        }

        for (warning_type, pattern) in &self.warning_patterns {
            if pattern.is_match(line) {
                summary.warning_counts
                    .entry(warning_type.clone())
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
                summary.total_warnings += 1;
            }
        }

        if line.contains("ERROR") {
            summary.error_lines.push(line.to_string());
        } else if line.contains("WARN") {
            summary.warning_lines.push(line.to_string());
        }
    }
}

#[derive(Debug, Default)]
pub struct LogSummary {
    pub total_lines: usize,
    pub total_errors: usize,
    pub total_warnings: usize,
    pub error_counts: HashMap<String, usize>,
    pub warning_counts: HashMap<String, usize>,
    pub error_lines: Vec<String>,
    pub warning_lines: Vec<String>,
}

impl LogSummary {
    pub fn new() -> Self {
        LogSummary {
            total_lines: 0,
            total_errors: 0,
            total_warnings: 0,
            error_counts: HashMap::new(),
            warning_counts: HashMap::new(),
            error_lines: Vec::new(),
            warning_lines: Vec::new(),
        }
    }

    pub fn print_summary(&self) {
        println!("Log Analysis Summary:");
        println!("Total lines processed: {}", self.total_lines);
        println!("Total errors found: {}", self.total_errors);
        println!("Total warnings found: {}", self.total_warnings);

        if !self.error_counts.is_empty() {
            println!("\nError breakdown:");
            for (error_type, count) in &self.error_counts {
                println!("  {}: {}", error_type, count);
            }
        }

        if !self.warning_counts.is_empty() {
            println!("\nWarning breakdown:");
            for (warning_type, count) in &self.warning_counts {
                println!("  {}: {}", warning_type, count);
            }
        }

        if !self.error_lines.is_empty() {
            println!("\nSample error lines:");
            for line in self.error_lines.iter().take(5) {
                println!("  {}", line);
            }
        }
    }
}