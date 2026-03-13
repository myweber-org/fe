use std::fs::File;
use std::io::{self, BufRead, BufReader};
use regex::Regex;

pub struct LogParser {
    error_pattern: Regex,
}

impl LogParser {
    pub fn new() -> Self {
        let pattern = r"ERROR|FATAL|CRITICAL|FAILED";
        let error_pattern = Regex::new(pattern).unwrap();
        LogParser { error_pattern }
    }

    pub fn parse_file(&self, file_path: &str) -> io::Result<Vec<String>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut errors = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if self.error_pattern.is_match(&line) {
                errors.push(format!("Line {}: {}", line_num + 1, line));
            }
        }

        Ok(errors)
    }

    pub fn count_errors(&self, file_path: &str) -> io::Result<usize> {
        let errors = self.parse_file(file_path)?;
        Ok(errors.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_file_with_errors() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "INFO: Application started").unwrap();
        writeln!(temp_file, "ERROR: Database connection failed").unwrap();
        writeln!(temp_file, "WARN: High memory usage").unwrap();
        writeln!(temp_file, "FATAL: System shutdown required").unwrap();

        let parser = LogParser::new();
        let errors = parser.parse_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(errors.len(), 2);
        assert!(errors[0].contains("ERROR: Database connection failed"));
        assert!(errors[1].contains("FATAL: System shutdown required"));
    }

    #[test]
    fn test_count_errors() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "INFO: Test message").unwrap();
        writeln!(temp_file, "ERROR: Something went wrong").unwrap();
        writeln!(temp_file, "CRITICAL: System failure").unwrap();

        let parser = LogParser::new();
        let count = parser.count_errors(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(count, 2);
    }
}