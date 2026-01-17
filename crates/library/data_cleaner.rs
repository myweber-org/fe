
use std::collections::HashSet;

pub struct DataCleaner {
    pub records: Vec<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            records: Vec::new(),
        }
    }

    pub fn add_record(&mut self, record: String) {
        self.records.push(record);
    }

    pub fn remove_duplicates(&mut self) -> usize {
        let original_count = self.records.len();
        let mut unique_set = HashSet::new();
        
        self.records.retain(|record| {
            if unique_set.contains(record) {
                false
            } else {
                unique_set.insert(record.clone());
                true
            }
        });
        
        original_count - self.records.len()
    }

    pub fn validate_records(&self) -> Vec<bool> {
        self.records
            .iter()
            .map(|record| {
                !record.trim().is_empty() && record.len() <= 1000
            })
            .collect()
    }

    pub fn get_valid_count(&self) -> usize {
        self.validate_records()
            .iter()
            .filter(|&&is_valid| is_valid)
            .count()
    }

    pub fn clean_all(&mut self) -> CleanResult {
        let duplicates_removed = self.remove_duplicates();
        let valid_count = self.get_valid_count();
        let total_count = self.records.len();
        
        CleanResult {
            total_records: total_count,
            valid_records: valid_count,
            duplicates_removed,
        }
    }
}

pub struct CleanResult {
    pub total_records: usize,
    pub valid_records: usize,
    pub duplicates_removed: usize,
}

impl CleanResult {
    pub fn is_clean(&self) -> bool {
        self.total_records == self.valid_records && self.duplicates_removed == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_duplicates() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("test".to_string());
        cleaner.add_record("test".to_string());
        cleaner.add_record("unique".to_string());
        
        let removed = cleaner.remove_duplicates();
        assert_eq!(removed, 1);
        assert_eq!(cleaner.records.len(), 2);
    }

    #[test]
    fn test_validate_records() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("valid".to_string());
        cleaner.add_record("".to_string());
        
        let validation = cleaner.validate_records();
        assert_eq!(validation, vec![true, false]);
    }
}
use std::collections::HashSet;
use std::io::{self, BufRead, Write};

pub fn clean_data(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let unique_lines: HashSet<&str> = lines.iter().cloned().collect();
    let mut sorted_lines: Vec<&str> = unique_lines.into_iter().collect();
    sorted_lines.sort();
    sorted_lines.join("\n")
}

pub fn process_stream() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut output = stdout.lock();
    
    let mut lines = Vec::new();
    for line in stdin.lock().lines() {
        lines.push(line?);
    }
    
    let cleaned = clean_data(&lines.join("\n"));
    writeln!(output, "{}", cleaned)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_data() {
        let input = "banana\napple\ncherry\napple\nbanana";
        let expected = "apple\nbanana\ncherry";
        assert_eq!(clean_data(input), expected);
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(clean_data(""), "");
    }
}