
use std::collections::HashMap;

pub struct DataCleaner {
    pub null_placeholder: String,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            null_placeholder: "N/A".to_string(),
        }
    }

    pub fn clean_vector(&self, data: Vec<Option<String>>) -> Vec<String> {
        data.into_iter()
            .map(|item| match item {
                Some(value) if !value.trim().is_empty() => value.trim().to_string(),
                _ => self.null_placeholder.clone(),
            })
            .collect()
    }

    pub fn normalize_strings(strings: &[String]) -> Vec<String> {
        strings
            .iter()
            .map(|s| s.to_lowercase().trim().to_string())
            .collect()
    }

    pub fn count_frequencies(items: &[String]) -> HashMap<String, usize> {
        let mut frequencies = HashMap::new();
        for item in items {
            *frequencies.entry(item.clone()).or_insert(0) += 1;
        }
        frequencies
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_vector() {
        let cleaner = DataCleaner::new();
        let data = vec![
            Some("  hello  ".to_string()),
            None,
            Some("".to_string()),
            Some("world".to_string()),
        ];
        
        let cleaned = cleaner.clean_vector(data);
        assert_eq!(cleaned, vec!["hello", "N/A", "N/A", "world"]);
    }

    #[test]
    fn test_normalize_strings() {
        let strings = vec![
            "  HELLO  ".to_string(),
            "World".to_string(),
            "  TEST  ".to_string(),
        ];
        
        let normalized = DataCleaner::normalize_strings(&strings);
        assert_eq!(normalized, vec!["hello", "world", "test"]);
    }

    #[test]
    fn test_count_frequencies() {
        let items = vec![
            "apple".to_string(),
            "banana".to_string(),
            "apple".to_string(),
            "orange".to_string(),
        ];
        
        let frequencies = DataCleaner::count_frequencies(&items);
        assert_eq!(frequencies.get("apple"), Some(&2));
        assert_eq!(frequencies.get("banana"), Some(&1));
        assert_eq!(frequencies.get("orange"), Some(&1));
    }
}