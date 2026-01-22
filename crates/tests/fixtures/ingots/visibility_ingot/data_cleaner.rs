use std::collections::HashSet;
use std::iter::FromIterator;

pub struct DataCleaner {
    records: Vec<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            records: Vec::new(),
        }
    }

    pub fn add_record(&mut self, record: &str) {
        self.records.push(record.to_string());
    }

    pub fn deduplicate(&mut self) -> Vec<String> {
        let unique_set: HashSet<String> = HashSet::from_iter(self.records.drain(..));
        let mut unique_vec: Vec<String> = unique_set.into_iter().collect();
        unique_vec.sort();
        self.records = unique_vec.clone();
        unique_vec
    }

    pub fn normalize_whitespace(&self) -> Vec<String> {
        self.records
            .iter()
            .map(|s| s.split_whitespace().collect::<Vec<&str>>().join(" "))
            .collect()
    }

    pub fn to_lowercase(&self) -> Vec<String> {
        self.records.iter().map(|s| s.to_lowercase()).collect()
    }

    pub fn count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("apple");
        cleaner.add_record("banana");
        cleaner.add_record("apple");
        cleaner.add_record("cherry");
        
        let unique = cleaner.deduplicate();
        assert_eq!(unique, vec!["apple", "banana", "cherry"]);
        assert_eq!(cleaner.count(), 3);
    }

    #[test]
    fn test_normalization() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("  multiple   spaces   here  ");
        cleaner.add_record("mixed\tTABS\nand newlines");
        
        let normalized = cleaner.normalize_whitespace();
        assert_eq!(normalized[0], "multiple spaces here");
        assert_eq!(normalized[1], "mixed TABS and newlines");
    }
}