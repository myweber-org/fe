use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use regex::Regex;

pub struct LogAnalyzer {
    error_pattern: Regex,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        let pattern = r"ERROR\s+\[(?P<module>[\w:]+)\]\s+(?P<message>.+)";
        let error_pattern = Regex::new(pattern).expect("Invalid regex pattern");
        LogAnalyzer { error_pattern }
    }

    pub fn analyze_file(&self, path: &str) -> io::Result<HashMap<String, usize>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut error_counts = HashMap::new();

        for line in reader.lines() {
            let line = line?;
            if let Some(captures) = self.error_pattern.captures(&line) {
                let module = captures.name("module").unwrap().as_str().to_string();
                *error_counts.entry(module).or_insert(0) += 1;
            }
        }

        Ok(error_counts)
    }

    pub fn generate_report(&self, counts: &HashMap<String, usize>) -> String {
        let mut report = String::from("Error Report:\n");
        let mut sorted_counts: Vec<_> = counts.iter().collect();
        sorted_counts.sort_by(|a, b| b.1.cmp(a.1));

        for (module, count) in sorted_counts {
            report.push_str(&format!("  {}: {} errors\n", module, count));
        }

        if counts.is_empty() {
            report.push_str("  No errors found\n");
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_error_parsing() {
        let analyzer = LogAnalyzer::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        
        writeln!(temp_file, "INFO [server] Starting up").unwrap();
        writeln!(temp_file, "ERROR [database:connection] Connection timeout").unwrap();
        writeln!(temp_file, "ERROR [api:auth] Invalid token").unwrap();
        writeln!(temp_file, "ERROR [database:connection] Query failed").unwrap();
        
        let counts = analyzer.analyze_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(counts.get("database:connection"), Some(&2));
        assert_eq!(counts.get("api:auth"), Some(&1));
        assert_eq!(counts.len(), 2);
    }
}