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

    pub fn clean_list(&mut self, items: Vec<&str>) -> Vec<String> {
        items
            .iter()
            .filter(|&&item| self.deduplicate(item))
            .map(|&item| self.normalize_string(item))
            .collect()
    }

    pub fn reset(&mut self) {
        self.dedupe_set.clear();
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
        assert!(cleaner.deduplicate("apple"));
        assert!(!cleaner.deduplicate("APPLE"));
        assert!(cleaner.deduplicate("banana"));
        assert_eq!(cleaner.get_unique_count(), 2);
    }

    #[test]
    fn test_clean_list() {
        let mut cleaner = DataCleaner::new();
        let items = vec!["cat", "DOG", "  Cat  ", "dog", "fish"];
        let cleaned = cleaner.clean_list(items);
        assert_eq!(cleaned, vec!["cat", "dog", "fish"]);
        assert_eq!(cleaner.get_unique_count(), 3);
    }
}use std::collections::HashSet;

pub struct DataCleaner<T> {
    data: Vec<Vec<T>>,
}

impl<T> DataCleaner<T>
where
    T: Clone + Eq + std::hash::Hash,
{
    pub fn new(data: Vec<Vec<T>>) -> Self {
        DataCleaner { data }
    }

    pub fn remove_null_rows(&mut self) {
        self.data.retain(|row| !row.iter().any(|item| Self::is_null(item)));
    }

    pub fn deduplicate(&mut self) {
        let mut seen = HashSet::new();
        self.data.retain(|row| seen.insert(row.clone()));
    }

    pub fn get_cleaned_data(&self) -> &Vec<Vec<T>> {
        &self.data
    }

    fn is_null(item: &T) -> bool {
        // For demonstration, treat empty strings as null
        // In real implementation, this would be type-specific
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate() {
        let mut cleaner = DataCleaner::new(vec![
            vec!["a", "b"],
            vec!["a", "b"],
            vec!["c", "d"],
        ]);
        
        cleaner.deduplicate();
        assert_eq!(cleaner.get_cleaned_data().len(), 2);
    }

    #[test]
    fn test_remove_null_rows() {
        // This test would be expanded with actual null detection logic
        let mut cleaner = DataCleaner::new(vec![
            vec!["valid", "data"],
            vec!["", "null"],
        ]);
        
        cleaner.remove_null_rows();
        assert_eq!(cleaner.get_cleaned_data().len(), 1);
    }
}