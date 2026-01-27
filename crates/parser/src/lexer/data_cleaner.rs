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
}use std::collections::HashSet;
use std::iter::FromIterator;

pub struct DataCleaner {
    data: Vec<String>,
}

impl DataCleaner {
    pub fn new(raw_data: Vec<String>) -> Self {
        DataCleaner { data: raw_data }
    }

    pub fn deduplicate(&mut self) -> &mut Self {
        let unique_set: HashSet<String> = HashSet::from_iter(self.data.drain(..));
        self.data = Vec::from_iter(unique_set.into_iter());
        self
    }

    pub fn normalize(&mut self) -> &mut Self {
        self.data = self.data
            .iter()
            .map(|s| s.trim().to_lowercase())
            .collect();
        self
    }

    pub fn sort_alphabetically(&mut self) -> &mut Self {
        self.data.sort();
        self
    }

    pub fn get_cleaned_data(&self) -> &Vec<String> {
        &self.data
    }

    pub fn process(&mut self) -> &Vec<String> {
        self.deduplicate()
            .normalize()
            .sort_alphabetically()
            .get_cleaned_data()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_cleaning_pipeline() {
        let raw_data = vec![
            "  Apple ".to_string(),
            "banana".to_string(),
            "Apple".to_string(),
            "  BANANA  ".to_string(),
            "Cherry".to_string(),
        ];
        
        let mut cleaner = DataCleaner::new(raw_data);
        let result = cleaner.process();
        
        assert_eq!(result, &vec!["apple", "banana", "cherry"]);
    }
}