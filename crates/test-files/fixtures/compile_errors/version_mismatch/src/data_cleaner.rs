
use std::collections::HashSet;

pub struct DataCleaner {
    records: Vec<String>,
    duplicates: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            records: Vec::new(),
            duplicates: HashSet::new(),
        }
    }

    pub fn add_record(&mut self, record: &str) -> bool {
        let trimmed = record.trim().to_string();
        
        if trimmed.is_empty() {
            return false;
        }

        if self.duplicates.contains(&trimmed) {
            return false;
        }

        self.duplicates.insert(trimmed.clone());
        self.records.push(trimmed);
        true
    }

    pub fn validate_records(&self) -> Vec<&String> {
        self.records
            .iter()
            .filter(|record| !record.contains("invalid"))
            .collect()
    }

    pub fn deduplicate(&mut self) -> usize {
        let original_count = self.records.len();
        self.duplicates.clear();
        
        let mut unique_records = Vec::new();
        for record in &self.records {
            if !self.duplicates.contains(record) {
                self.duplicates.insert(record.clone());
                unique_records.push(record.clone());
            }
        }
        
        self.records = unique_records;
        original_count - self.records.len()
    }

    pub fn get_records(&self) -> &Vec<String> {
        &self.records
    }

    pub fn clear(&mut self) {
        self.records.clear();
        self.duplicates.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_record() {
        let mut cleaner = DataCleaner::new();
        assert!(cleaner.add_record("valid_data"));
        assert!(!cleaner.add_record("valid_data"));
        assert!(!cleaner.add_record(""));
        assert!(!cleaner.add_record("   "));
    }

    #[test]
    fn test_validate_records() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("good_record");
        cleaner.add_record("invalid_record");
        cleaner.add_record("another_good");
        
        let valid = cleaner.validate_records();
        assert_eq!(valid.len(), 2);
        assert!(valid.iter().all(|r| !r.contains("invalid")));
    }

    #[test]
    fn test_deduplicate() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("duplicate");
        cleaner.add_record("duplicate");
        cleaner.add_record("unique");
        
        let removed = cleaner.deduplicate();
        assert_eq!(removed, 1);
        assert_eq!(cleaner.get_records().len(), 2);
    }
}