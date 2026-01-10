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
        
        for record in &self.records {
            if unique_set.insert(record.clone()) {
                deduped_records.push(record.clone());
            }
        }
        
        let removed_count = self.records.len() - deduped_records.len();
        self.records = deduped_records;
        removed_count
    }

    pub fn validate_records(&self) -> (usize, usize) {
        let mut valid_count = 0;
        
        for record in &self.records {
            if !record.trim().is_empty() && record.len() <= 100 {
                valid_count += 1;
            }
        }
        
        (valid_count, self.records.len() - valid_count)
    }

    pub fn get_records(&self) -> &Vec<String> {
        &self.records
    }

    pub fn clear(&mut self) {
        self.records.clear();
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
        assert_eq!(cleaner.get_records().len(), 2);
    }

    #[test]
    fn test_validation() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("valid".to_string());
        cleaner.add_record("".to_string());
        cleaner.add_record("x".repeat(101));
        
        let (valid, invalid) = cleaner.validate_records();
        assert_eq!(valid, 1);
        assert_eq!(invalid, 2);
    }
}use std::collections::HashSet;

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
        let original_len = self.records.len();
        let mut seen = HashSet::new();
        
        self.records.retain(|record| {
            if seen.contains(record) {
                false
            } else {
                seen.insert(record.clone());
                true
            }
        });
        
        original_len - self.records.len()
    }

    pub fn validate_records(&self) -> Vec<bool> {
        self.records
            .iter()
            .map(|record| {
                !record.trim().is_empty() 
                && record.len() <= 1000
                && !record.contains("NULL")
            })
            .collect()
    }

    pub fn get_valid_records(&self) -> Vec<String> {
        let validation = self.validate_records();
        self.records
            .iter()
            .enumerate()
            .filter(|(i, _)| validation[*i])
            .map(|(_, record)| record.clone())
            .collect()
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
        cleaner.add_record("x".repeat(1001));
        
        let valid = cleaner.get_valid_records();
        assert_eq!(valid.len(), 1);
        assert_eq!(valid[0], "valid");
    }
}
use std::collections::HashSet;

pub struct DataCleaner {
    dedupe_set: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            dedupe_set: HashSet::new(),
        }
    }

    pub fn normalize_text(&self, text: &str) -> String {
        text.trim()
            .to_lowercase()
            .replace(|c: char| !c.is_alphanumeric() && c != ' ', "")
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    }

    pub fn deduplicate(&mut self, item: &str) -> bool {
        let normalized = self.normalize_text(item);
        if self.dedupe_set.contains(&normalized) {
            false
        } else {
            self.dedupe_set.insert(normalized);
            true
        }
    }

    pub fn batch_process(&mut self, items: Vec<String>) -> Vec<String> {
        items
            .into_iter()
            .filter(|item| self.deduplicate(item))
            .collect()
    }

    pub fn get_unique_count(&self) -> usize {
        self.dedupe_set.len()
    }

    pub fn clear(&mut self) {
        self.dedupe_set.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalization() {
        let cleaner = DataCleaner::new();
        let result = cleaner.normalize_text("  Hello, World!  ");
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        assert!(cleaner.deduplicate("Hello World"));
        assert!(!cleaner.deduplicate("hello world"));
        assert!(cleaner.deduplicate("Another Item"));
    }

    #[test]
    fn test_batch_processing() {
        let mut cleaner = DataCleaner::new();
        let items = vec![
            "Item One".to_string(),
            "item one".to_string(),
            "ITEM TWO".to_string(),
        ];
        let result = cleaner.batch_process(items);
        assert_eq!(result.len(), 2);
        assert_eq!(cleaner.get_unique_count(), 2);
    }
}