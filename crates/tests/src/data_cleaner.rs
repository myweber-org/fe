
use std::collections::HashSet;

pub struct DataCleaner {
    entries: Vec<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            entries: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, entry: &str) {
        let normalized = entry.trim().to_lowercase();
        self.entries.push(normalized);
    }

    pub fn clean(&mut self) -> Vec<String> {
        let unique_set: HashSet<String> = self.entries.drain(..).collect();
        let mut result: Vec<String> = unique_set.into_iter().collect();
        result.sort();
        result
    }

    pub fn process_raw_data(raw_data: &[&str]) -> Vec<String> {
        let mut cleaner = DataCleaner::new();
        for item in raw_data {
            cleaner.add_entry(item);
        }
        cleaner.clean()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duplicate_removal() {
        let raw_data = vec!["Apple", "apple", "APPLE", "Banana", "banana"];
        let cleaned = DataCleaner::process_raw_data(&raw_data);
        assert_eq!(cleaned, vec!["apple", "banana"]);
    }

    #[test]
    fn test_sorting() {
        let raw_data = vec!["Zebra", "apple", "Banana"];
        let cleaned = DataCleaner::process_raw_data(&raw_data);
        assert_eq!(cleaned, vec!["apple", "banana", "zebra"]);
    }
}