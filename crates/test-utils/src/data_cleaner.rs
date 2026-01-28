
use std::collections::HashSet;

pub struct DataCleaner {
    unique_items: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            unique_items: HashSet::new(),
        }
    }

    pub fn add_item(&mut self, item: &str) -> bool {
        let normalized = Self::normalize_string(item);
        self.unique_items.insert(normalized)
    }

    pub fn get_unique_items(&self) -> Vec<String> {
        let mut items: Vec<String> = self.unique_items.iter().cloned().collect();
        items.sort();
        items
    }

    pub fn clear(&mut self) {
        self.unique_items.clear();
    }

    pub fn count(&self) -> usize {
        self.unique_items.len()
    }

    fn normalize_string(s: &str) -> String {
        s.trim().to_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let mut cleaner = DataCleaner::new();
        assert_eq!(cleaner.count(), 0);

        assert!(cleaner.add_item("Apple"));
        assert!(cleaner.add_item("Banana"));
        assert!(!cleaner.add_item("apple"));
        assert!(!cleaner.add_item("  APPLE  "));

        assert_eq!(cleaner.count(), 2);

        let items = cleaner.get_unique_items();
        assert_eq!(items, vec!["apple", "banana"]);

        cleaner.clear();
        assert_eq!(cleaner.count(), 0);
    }

    #[test]
    fn test_normalization() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_item("  Mixed CASE  ");
        cleaner.add_item("MIXED case");
        cleaner.add_item("mixed case");

        assert_eq!(cleaner.count(), 1);
        assert_eq!(cleaner.get_unique_items(), vec!["mixed case"]);
    }
}