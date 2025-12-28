
use std::collections::HashSet;

pub struct DataCleaner {
    processed_items: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            processed_items: HashSet::new(),
        }
    }

    pub fn clean_string(&mut self, input: &str) -> Option<String> {
        let normalized = input.trim().to_lowercase();
        
        if normalized.is_empty() {
            return None;
        }

        if self.processed_items.contains(&normalized) {
            return None;
        }

        self.processed_items.insert(normalized.clone());
        Some(normalized)
    }

    pub fn process_batch(&mut self, inputs: &[&str]) -> Vec<String> {
        inputs
            .iter()
            .filter_map(|&input| self.clean_string(input))
            .collect()
    }

    pub fn get_unique_count(&self) -> usize {
        self.processed_items.len()
    }

    pub fn reset(&mut self) {
        self.processed_items.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_string() {
        let mut cleaner = DataCleaner::new();
        
        assert_eq!(cleaner.clean_string("  HELLO  "), Some("hello".to_string()));
        assert_eq!(cleaner.clean_string("hello"), None);
        assert_eq!(cleaner.clean_string("   "), None);
    }

    #[test]
    fn test_process_batch() {
        let mut cleaner = DataCleaner::new();
        let inputs = vec!["Apple", "apple", "BANANA", "  banana  ", ""];
        let result = cleaner.process_batch(&inputs);
        
        assert_eq!(result.len(), 2);
        assert_eq!(cleaner.get_unique_count(), 2);
    }
}