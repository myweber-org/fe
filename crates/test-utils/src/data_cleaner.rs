use std::collections::HashSet;

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
        let mut unique_set = HashSet::new();
        let mut deduped_records = Vec::new();
        
        for record in self.records.drain(..) {
            if unique_set.insert(record.clone()) {
                deduped_records.push(record);
            }
        }
        
        let removed_count = self.records.len() - deduped_records.len();
        self.records = deduped_records;
        removed_count
    }

    pub fn validate_records(&self) -> Vec<bool> {
        self.records
            .iter()
            .map(|record| {
                !record.trim().is_empty() 
                && record.len() <= 1000 
                && !record.contains('\0')
            })
            .collect()
    }

    pub fn get_valid_records(&self) -> Vec<&String> {
        let validation_results = self.validate_records();
        self.records
            .iter()
            .enumerate()
            .filter(|(i, _)| validation_results[*i])
            .map(|(_, record)| record)
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
        
        let removed = cleaner.deduplicate();
        assert_eq!(removed, 1);
        assert_eq!(cleaner.record_count(), 2);
    }

    #[test]
    fn test_validation() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("valid".to_string());
        cleaner.add_record("".to_string());
        cleaner.add_record("x".repeat(1001));
        
        let valid_records = cleaner.get_valid_records();
        assert_eq!(valid_records.len(), 1);
        assert_eq!(*valid_records[0], "valid");
    }
}