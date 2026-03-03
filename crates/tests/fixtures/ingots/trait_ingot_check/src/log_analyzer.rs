use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

#[derive(Debug, PartialEq, Eq, Hash)]
enum LogLevel {
    Error,
    Warning,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "ERROR" => Some(LogLevel::Error),
            "WARNING" => Some(LogLevel::Warning),
            "INFO" => Some(LogLevel::Info),
            "DEBUG" => Some(LogLevel::Debug),
            "TRACE" => Some(LogLevel::Trace),
            _ => None,
        }
    }
}

pub struct LogAnalyzer {
    counts: HashMap<LogLevel, u32>,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            counts: HashMap::new(),
        }
    }

    pub fn analyze_file<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let file = File::open(path)?;
        let reader = io::BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            self.process_line(&line);
        }

        Ok(())
    }

    fn process_line(&mut self, line: &str) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            if let Some(level) = LogLevel::from_str(parts[0]) {
                *self.counts.entry(level).or_insert(0) += 1;
            }
        }
    }

    pub fn get_counts(&self) -> &HashMap<LogLevel, u32> {
        &self.counts
    }

    pub fn total_errors(&self) -> u32 {
        self.counts.get(&LogLevel::Error).copied().unwrap_or(0)
    }

    pub fn summary(&self) -> String {
        let mut result = String::from("Log Analysis Summary:\n");
        let levels = [
            LogLevel::Error,
            LogLevel::Warning,
            LogLevel::Info,
            LogLevel::Debug,
            LogLevel::Trace,
        ];

        for level in levels.iter() {
            if let Some(count) = self.counts.get(level) {
                result.push_str(&format!("{:?}: {}\n", level, count));
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_analyzer() {
        let mut analyzer = LogAnalyzer::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "ERROR Database connection failed").unwrap();
        writeln!(temp_file, "WARNING High memory usage").unwrap();
        writeln!(temp_file, "INFO User logged in").unwrap();
        writeln!(temp_file, "ERROR File not found").unwrap();
        writeln!(temp_file, "INFO Request processed").unwrap();

        analyzer.analyze_file(temp_file.path()).unwrap();
        
        let counts = analyzer.get_counts();
        assert_eq!(counts.get(&LogLevel::Error), Some(&2));
        assert_eq!(counts.get(&LogLevel::Warning), Some(&1));
        assert_eq!(counts.get(&LogLevel::Info), Some(&2));
        assert_eq!(analyzer.total_errors(), 2);
    }

    #[test]
    fn test_invalid_log_level() {
        let mut analyzer = LogAnalyzer::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "UNKNOWN Some message").unwrap();
        writeln!(temp_file, "INFO Valid message").unwrap();

        analyzer.analyze_file(temp_file.path()).unwrap();
        
        let counts = analyzer.get_counts();
        assert_eq!(counts.get(&LogLevel::Info), Some(&1));
        assert_eq!(counts.len(), 1);
    }
}