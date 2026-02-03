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

    pub fn analyze_file(&self, path: &str) -> Result<HashMap<String, usize>, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);
        
        let mut counts = HashMap::new();
        counts.insert("ERROR".to_string(), 0);
        counts.insert("WARN".to_string(), 0);
        counts.insert("INFO".to_string(), 0);
        counts.insert("TOTAL".to_string(), 0);

        for line_result in reader.lines() {
            let line = line_result.map_err(|e| format!("Failed to read line: {}", e))?;
            
            *counts.get_mut("TOTAL").unwrap() += 1;
            
            if self.error_pattern.is_match(&line) {
                *counts.get_mut("ERROR").unwrap() += 1;
            } else if self.warn_pattern.is_match(&line) {
                *counts.get_mut("WARN").unwrap() += 1;
            } else if self.info_pattern.is_match(&line) {
                *counts.get_mut("INFO").unwrap() += 1;
            }
        }

        Ok(counts)
    }

    pub fn generate_report(&self, counts: &HashMap<String, usize>) -> String {
        let total = counts.get("TOTAL").unwrap_or(&0);
        let errors = counts.get("ERROR").unwrap_or(&0);
        let warnings = counts.get("WARN").unwrap_or(&0);
        let infos = counts.get("INFO").unwrap_or(&0);

        format!(
            "Log Analysis Report:\n\
             Total entries: {}\n\
             Errors: {} ({:.1}%)\n\
             Warnings: {} ({:.1}%)\n\
             Info messages: {} ({:.1}%)",
            total,
            errors,
            (*errors as f64 / *total as f64) * 100.0,
            warnings,
            (*warnings as f64 / *total as f64) * 100.0,
            infos,
            (*infos as f64 / *total as f64) * 100.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_analysis() {
        let analyzer = LogAnalyzer::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "INFO: Application started").unwrap();
        writeln!(temp_file, "WARN: Disk space low").unwrap();
        writeln!(temp_file, "ERROR: Database connection failed").unwrap();
        writeln!(temp_file, "INFO: User login successful").unwrap();
        
        let counts = analyzer.analyze_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(counts.get("TOTAL"), Some(&4));
        assert_eq!(counts.get("ERROR"), Some(&1));
        assert_eq!(counts.get("WARN"), Some(&1));
        assert_eq!(counts.get("INFO"), Some(&2));
    }
}