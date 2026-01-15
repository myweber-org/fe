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
        let mut unique_set = HashSet::new();
        let mut deduped_records = Vec::new();
        let initial_count = self.records.len();

        for record in self.records.drain(..) {
            if unique_set.insert(record.clone()) {
                deduped_records.push(record);
            }
        }

        self.records = deduped_records;
        initial_count - self.records.len()
    }

    pub fn validate_records(&self) -> (usize, usize) {
        let mut valid_count = 0;
        let mut invalid_count = 0;

        for record in &self.records {
            if !record.trim().is_empty() && record.len() <= 1000 {
                valid_count += 1;
            } else {
                invalid_count += 1;
            }
        }

        (valid_count, invalid_count)
    }

    pub fn get_statistics(&self) -> (usize, usize, f64) {
        let total = self.records.len();
        let total_chars: usize = self.records.iter().map(|r| r.len()).sum();
        let avg_length = if total > 0 {
            total_chars as f64 / total as f64
        } else {
            0.0
        };

        (total, total_chars, avg_length)
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

        let removed = cleaner.remove_duplicates();
        assert_eq!(removed, 1);
        assert_eq!(cleaner.records.len(), 2);
    }

    #[test]
    fn test_validation() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("valid".to_string());
        cleaner.add_record("".to_string());

        let (valid, invalid) = cleaner.validate_records();
        assert_eq!(valid, 1);
        assert_eq!(invalid, 1);
    }
}