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

    pub fn normalize_string(&self, input: &str) -> String {
        input.trim().to_lowercase()
    }

    pub fn deduplicate(&mut self, item: &str) -> bool {
        let normalized = self.normalize_string(item);
        if self.dedupe_set.contains(&normalized) {
            false
        } else {
            self.dedupe_set.insert(normalized);
            true
        }
    }

    pub fn clean_data(&mut self, data: Vec<&str>) -> Vec<String> {
        let mut cleaned = Vec::new();
        
        for item in data {
            if self.deduplicate(item) {
                cleaned.push(self.normalize_string(item));
            }
        }
        
        cleaned
    }

    pub fn get_unique_count(&self) -> usize {
        self.dedupe_set.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalization() {
        let cleaner = DataCleaner::new();
        assert_eq!(cleaner.normalize_string("  HELLO World  "), "hello world");
    }

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        assert!(cleaner.deduplicate("test"));
        assert!(!cleaner.deduplicate("TEST"));
        assert!(cleaner.deduplicate("another"));
    }

    #[test]
    fn test_clean_data() {
        let mut cleaner = DataCleaner::new();
        let data = vec!["Apple", "  apple ", "BANANA", "banana", "Cherry"];
        let cleaned = cleaner.clean_data(data);
        
        assert_eq!(cleaned.len(), 3);
        assert_eq!(cleaner.get_unique_count(), 3);
    }
}