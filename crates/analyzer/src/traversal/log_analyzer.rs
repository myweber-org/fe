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
    pub unique_errors: HashMap<String, usize>,
    pub time_range: Option<(String, String)>,
}

impl LogSummary {
    pub fn new() -> Self {
        LogSummary {
            total_lines: 0,
            error_count: 0,
            warning_count: 0,
            info_count: 0,
            unique_errors: HashMap::new(),
            time_range: None,
        }
    }

    pub fn analyze_file<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
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

        let line_lower = line.to_lowercase();
        
        if line_lower.contains("error") {
            self.error_count += 1;
            self.extract_error_pattern(line);
        } else if line_lower.contains("warning") {
            self.warning_count += 1;
        } else if line_lower.contains("info") {
            self.info_count += 1;
        }

        self.extract_timestamp(line);
    }

    fn extract_error_pattern(&mut self, line: &str) {
        let error_key = line
            .split_whitespace()
            .skip_while(|word| !word.to_lowercase().contains("error"))
            .take(3)
            .collect::<Vec<&str>>()
            .join(" ");

        if !error_key.is_empty() {
            *self.unique_errors.entry(error_key).or_insert(0) += 1;
        }
    }

    fn extract_timestamp(&mut self, line: &str) {
        if let Some(first_word) = line.split_whitespace().next() {
            if first_word.contains(':') && first_word.chars().any(|c| c.is_digit(10)) {
                match &mut self.time_range {
                    None => {
                        self.time_range = Some((first_word.to_string(), first_word.to_string()));
                    }
                    Some((start, end)) => {
                        if first_word < *start {
                            *start = first_word.to_string();
                        }
                        if first_word > *end {
                            *end = first_word.to_string();
                        }
                    }
                }
            }
        }
    }

    pub fn print_summary(&self) {
        println!("Log Analysis Summary:");
        println!("=====================");
        println!("Total lines processed: {}", self.total_lines);
        println!("Error count: {}", self.error_count);
        println!("Warning count: {}", self.warning_count);
        println!("Info count: {}", self.info_count);
        
        if let Some((start, end)) = &self.time_range {
            println!("Time range: {} - {}", start, end);
        }
        
        if !self.unique_errors.is_empty() {
            println!("\nUnique error patterns:");
            for (pattern, count) in &self.unique_errors {
                println!("  {}: {}", pattern, count);
            }
        }
        
        let error_percentage = if self.total_lines > 0 {
            (self.error_count as f64 / self.total_lines as f64) * 100.0
        } else {
            0.0
        };
        println!("\nError percentage: {:.2}%", error_percentage);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_analysis() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "2023-10-01 10:00:00 INFO Application started").unwrap();
        writeln!(temp_file, "2023-10-01 10:01:00 ERROR Database connection failed").unwrap();
        writeln!(temp_file, "2023-10-01 10:02:00 WARNING High memory usage").unwrap();
        writeln!(temp_file, "2023-10-01 10:03:00 ERROR Database connection failed").unwrap();
        
        let summary = LogSummary::analyze_file(temp_file.path()).unwrap();
        
        assert_eq!(summary.total_lines, 4);
        assert_eq!(summary.error_count, 2);
        assert_eq!(summary.warning_count, 1);
        assert_eq!(summary.info_count, 1);
        assert_eq!(summary.unique_errors.len(), 1);
    }
}