use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;

#[derive(Debug)]
pub struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    source: String,
}

pub struct LogAnalyzer {
    entries: Vec<LogEntry>,
    level_counts: HashMap<String, usize>,
    source_counts: HashMap<String, usize>,
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            entries: Vec::new(),
            level_counts: HashMap::new(),
            source_counts: HashMap::new(),
        }
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let log_pattern = Regex::new(r"\[(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2})\] (\w+) \[(\w+)\]: (.+)")?;

        for line in reader.lines() {
            let line = line?;
            if let Some(captures) = log_pattern.captures(&line) {
                let entry = LogEntry {
                    timestamp: captures[1].to_string(),
                    level: captures[2].to_string(),
                    source: captures[3].to_string(),
                    message: captures[4].to_string(),
                };

                *self.level_counts.entry(entry.level.clone()).or_insert(0) += 1;
                *self.source_counts.entry(entry.source.clone()).or_insert(0) += 1;
                self.entries.push(entry);
            }
        }

        Ok(())
    }

    pub fn filter_by_level(&self, level: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.level == level)
            .collect()
    }

    pub fn filter_by_source(&self, source: &str) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.source == source)
            .collect()
    }

    pub fn get_summary(&self) -> HashMap<String, usize> {
        let mut summary = HashMap::new();
        summary.insert("total_entries".to_string(), self.entries.len());
        summary.insert("unique_levels".to_string(), self.level_counts.len());
        summary.insert("unique_sources".to_string(), self.source_counts.len());

        for (level, count) in &self.level_counts {
            summary.insert(format!("level_{}", level), *count);
        }

        for (source, count) in &self.source_counts {
            summary.insert(format!("source_{}", source), *count);
        }

        summary
    }

    pub fn find_pattern(&self, pattern: &str) -> Vec<&LogEntry> {
        let re = Regex::new(pattern).unwrap_or_else(|_| Regex::new(".*").unwrap());
        self.entries
            .iter()
            .filter(|entry| re.is_match(&entry.message))
            .collect()
    }
}

impl Default for LogAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_analyzer() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "[2024-01-15 10:30:45] INFO [API]: Request received").unwrap();
        writeln!(temp_file, "[2024-01-15 10:30:46] ERROR [DB]: Connection failed").unwrap();
        writeln!(temp_file, "[2024-01-15 10:30:47] WARN [CACHE]: Memory threshold exceeded").unwrap();

        let mut analyzer = LogAnalyzer::new();
        analyzer.load_from_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(analyzer.entries.len(), 3);
        assert_eq!(analyzer.filter_by_level("ERROR").len(), 1);
        assert_eq!(analyzer.filter_by_source("API").len(), 1);
        
        let summary = analyzer.get_summary();
        assert_eq!(summary.get("total_entries"), Some(&3));
    }
}