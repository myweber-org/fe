use std::collections::HashSet;
use std::io::{self, BufRead, Write};

fn clean_data(input: Vec<String>) -> Vec<String> {
    let mut unique_items: HashSet<String> = input.into_iter().collect();
    let mut sorted_items: Vec<String> = unique_items.into_iter().collect();
    sorted_items.sort();
    sorted_items
}

fn read_input() -> io::Result<Vec<String>> {
    let stdin = io::stdin();
    let mut lines = Vec::new();
    
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            break;
        }
        lines.push(line.trim().to_string());
    }
    
    Ok(lines)
}

fn write_output(cleaned_data: &[String]) -> io::Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    
    for item in cleaned_data {
        writeln!(handle, "{}", item)?;
    }
    
    Ok(())
}

fn main() -> io::Result<()> {
    let input_data = read_input()?;
    let cleaned_data = clean_data(input_data);
    write_output(&cleaned_data)?;
    Ok(())
}use std::collections::HashSet;

pub struct DataCleaner {
    records: Vec<String>,
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

    pub fn deduplicate(&mut self) -> usize {
        let unique_set: HashSet<String> = self.records.drain(..).collect();
        let original_count = self.records.len();
        self.records = unique_set.into_iter().collect();
        original_count - self.records.len()
    }

    pub fn validate_records(&self) -> Vec<bool> {
        self.records
            .iter()
            .map(|record| !record.trim().is_empty() && record.len() <= 255)
            .collect()
    }

    pub fn get_valid_records(&self) -> Vec<&String> {
        self.records
            .iter()
            .filter(|record| !record.trim().is_empty() && record.len() <= 255)
            .collect()
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("test".to_string());
        cleaner.add_record("test".to_string());
        cleaner.add_record("unique".to_string());
        
        let duplicates_removed = cleaner.deduplicate();
        assert_eq!(duplicates_removed, 1);
        assert_eq!(cleaner.record_count(), 2);
    }

    #[test]
    fn test_validation() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("valid".to_string());
        cleaner.add_record("".to_string());
        cleaner.add_record("x".repeat(256));
        
        let validation_results = cleaner.validate_records();
        assert_eq!(validation_results, vec![true, false, false]);
    }
}