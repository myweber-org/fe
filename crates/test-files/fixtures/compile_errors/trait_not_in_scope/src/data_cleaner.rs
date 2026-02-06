use std::collections::HashSet;
use std::hash::Hash;

pub struct DataCleaner<T> {
    seen: HashSet<T>,
}

impl<T> DataCleaner<T>
where
    T: Eq + Hash + Clone,
{
    pub fn new() -> Self {
        DataCleaner {
            seen: HashSet::new(),
        }
    }

    pub fn deduplicate(&mut self, items: Vec<T>) -> Vec<T> {
        let mut result = Vec::new();
        for item in items {
            if self.seen.insert(item.clone()) {
                result.push(item);
            }
        }
        result
    }

    pub fn normalize_strings(strings: Vec<String>) -> Vec<String> {
        strings
            .into_iter()
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect()
    }

    pub fn reset(&mut self) {
        self.seen.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate_integers() {
        let mut cleaner = DataCleaner::new();
        let input = vec![1, 2, 2, 3, 4, 4, 4, 5];
        let result = cleaner.deduplicate(input);
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_deduplicate_strings() {
        let mut cleaner = DataCleaner::new();
        let input = vec!["apple", "banana", "apple", "cherry"]
            .into_iter()
            .map(String::from)
            .collect();
        let result = cleaner.deduplicate(input);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_normalize_strings() {
        let input = vec![
            "  Hello  ".to_string(),
            "WORLD".to_string(),
            "".to_string(),
            "  Rust  ".to_string(),
        ];
        let result = DataCleaner::normalize_strings(input);
        assert_eq!(result, vec!["hello", "world", "rust"]);
    }

    #[test]
    fn test_reset() {
        let mut cleaner = DataCleaner::new();
        let input1 = vec![1, 2, 3];
        cleaner.deduplicate(input1);
        assert_eq!(cleaner.seen.len(), 3);
        
        cleaner.reset();
        assert!(cleaner.seen.is_empty());
        
        let input2 = vec![1, 2, 3];
        let result = cleaner.deduplicate(input2);
        assert_eq!(result, vec![1, 2, 3]);
    }
}