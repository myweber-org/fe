
use std::collections::HashSet;

pub struct DataCleaner {
    pub data: Vec<String>,
}

impl DataCleaner {
    pub fn new(data: Vec<String>) -> Self {
        DataCleaner { data }
    }

    pub fn remove_duplicates(&mut self) {
        let unique_set: HashSet<String> = self.data.drain(..).collect();
        self.data = unique_set.into_iter().collect();
    }

    pub fn normalize_strings(&mut self) {
        for item in &mut self.data {
            *item = item.trim().to_lowercase();
        }
    }

    pub fn clean(&mut self) -> &Vec<String> {
        self.remove_duplicates();
        self.normalize_strings();
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_cleaner() {
        let raw_data = vec![
            "  Apple  ".to_string(),
            "banana".to_string(),
            "  APPLE  ".to_string(),
            "Banana".to_string(),
            "cherry".to_string(),
        ];

        let mut cleaner = DataCleaner::new(raw_data);
        let cleaned = cleaner.clean();

        assert_eq!(cleaned.len(), 3);
        assert!(cleaned.contains(&"apple".to_string()));
        assert!(cleaned.contains(&"banana".to_string()));
        assert!(cleaned.contains(&"cherry".to_string()));
    }
}use std::collections::HashSet;

pub struct DataCleaner {
    dedupe_set: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            dedupe_set: HashSet::new(),
        }
    }

    pub fn normalize_text(&self, input: &str) -> String {
        input.trim().to_lowercase()
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

    pub fn process_batch(&mut self, items: Vec<&str>) -> Vec<String> {
        items
            .iter()
            .filter(|&&item| self.deduplicate(item))
            .map(|&item| self.normalize_text(item))
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
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        let data = vec!["Apple", "apple", "APPLE", "Banana", "banana"];
        let result = cleaner.process_batch(data);
        assert_eq!(result.len(), 2);
        assert_eq!(cleaner.get_unique_count(), 2);
    }

    #[test]
    fn test_normalization() {
        let cleaner = DataCleaner::new();
        assert_eq!(cleaner.normalize_text("  HELLO World  "), "hello world");
    }
}
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

    pub fn add_record(&mut self, record: String) -> bool {
        if self.duplicates.contains(&record) {
            return false;
        }
        
        if self.records.contains(&record) {
            self.duplicates.insert(record.clone());
            return false;
        }
        
        self.records.push(record);
        true
    }

    pub fn validate_records(&self) -> Vec<&String> {
        self.records
            .iter()
            .filter(|record| !record.trim().is_empty())
            .collect()
    }

    pub fn get_unique_count(&self) -> usize {
        self.records.len()
    }

    pub fn get_duplicate_count(&self) -> usize {
        self.duplicates.len()
    }

    pub fn clear_duplicates(&mut self) {
        self.duplicates.clear();
    }
}

pub fn sanitize_input(input: &str) -> String {
    input
        .trim()
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        assert!(cleaner.add_record("test1".to_string()));
        assert!(!cleaner.add_record("test1".to_string()));
        assert_eq!(cleaner.get_unique_count(), 1);
        assert_eq!(cleaner.get_duplicate_count(), 1);
    }

    #[test]
    fn test_validation() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("  ".to_string());
        cleaner.add_record("valid".to_string());
        let valid = cleaner.validate_records();
        assert_eq!(valid.len(), 1);
        assert_eq!(valid[0], "valid");
    }

    #[test]
    fn test_sanitize() {
        let result = sanitize_input("  TEST@123  ");
        assert_eq!(result, "test123");
    }
}