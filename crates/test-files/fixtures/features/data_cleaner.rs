use std::collections::HashSet;

pub struct DataCleaner {
    records: Vec<String>,
    unique_set: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            records: Vec::new(),
            unique_set: HashSet::new(),
        }
    }

    pub fn add_record(&mut self, record: &str) -> bool {
        let trimmed = record.trim().to_string();
        
        if trimmed.is_empty() {
            return false;
        }

        if self.unique_set.insert(trimmed.clone()) {
            self.records.push(trimmed);
            true
        } else {
            false
        }
    }

    pub fn validate_records(&self) -> Vec<&String> {
        self.records
            .iter()
            .filter(|record| record.len() > 3 && record.len() < 256)
            .collect()
    }

    pub fn get_clean_records(&self) -> Vec<String> {
        self.validate_records()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }

    pub fn clear(&mut self) {
        self.records.clear();
        self.unique_set.clear();
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
        
        assert!(cleaner.add_record("test"));
        assert!(!cleaner.add_record("test"));
        assert!(cleaner.add_record("another"));
        
        assert_eq!(cleaner.record_count(), 2);
    }

    #[test]
    fn test_validation() {
        let mut cleaner = DataCleaner::new();
        
        cleaner.add_record("abc");
        cleaner.add_record("valid_record");
        cleaner.add_record("x");
        
        let valid = cleaner.validate_records();
        assert_eq!(valid.len(), 1);
        assert_eq!(valid[0], "valid_record");
    }

    #[test]
    fn test_empty_record() {
        let mut cleaner = DataCleaner::new();
        
        assert!(!cleaner.add_record(""));
        assert!(!cleaner.add_record("   "));
        assert_eq!(cleaner.record_count(), 0);
    }
}