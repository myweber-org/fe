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

    pub fn clean_dataset(&mut self, data: Vec<&str>) -> Vec<String> {
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
        assert_eq!(cleaner.normalize_string("  TEST Data  "), "test data");
    }

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        assert!(cleaner.deduplicate("Apple"));
        assert!(!cleaner.deduplicate("apple "));
        assert!(cleaner.deduplicate("Banana"));
        assert_eq!(cleaner.get_unique_count(), 2);
    }

    #[test]
    fn test_clean_dataset() {
        let mut cleaner = DataCleaner::new();
        let data = vec!["Apple", " apple ", "Banana", "BANANA", "Cherry"];
        let result = cleaner.clean_dataset(data);
        assert_eq!(result.len(), 3);
        assert!(result.contains(&"apple".to_string()));
        assert!(result.contains(&"banana".to_string()));
        assert!(result.contains(&"cherry".to_string()));
    }
}