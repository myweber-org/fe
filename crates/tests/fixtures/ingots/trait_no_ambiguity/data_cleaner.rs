use std::collections::HashSet;

pub struct DataCleaner {
    deduplication_enabled: bool,
    normalization_enabled: bool,
}

impl DataCleaner {
    pub fn new(deduplication: bool, normalization: bool) -> Self {
        DataCleaner {
            deduplication_enabled: deduplication,
            normalization_enabled: normalization,
        }
    }

    pub fn clean_dataset(&self, data: Vec<String>) -> Vec<String> {
        let mut processed_data = data;

        if self.deduplication_enabled {
            processed_data = Self::remove_duplicates(processed_data);
        }

        if self.normalization_enabled {
            processed_data = Self::normalize_entries(processed_data);
        }

        processed_data
    }

    fn remove_duplicates(data: Vec<String>) -> Vec<String> {
        let mut seen = HashSet::new();
        data.into_iter()
            .filter(|item| seen.insert(item.clone()))
            .collect()
    }

    fn normalize_entries(data: Vec<String>) -> Vec<String> {
        data.into_iter()
            .map(|entry| {
                entry.trim()
                    .to_lowercase()
                    .chars()
                    .filter(|c| c.is_alphanumeric() || c.is_whitespace())
                    .collect()
            })
            .collect()
    }

    pub fn statistics(&self, original: &[String], cleaned: &[String]) -> (usize, usize, f64) {
        let original_len = original.len();
        let cleaned_len = cleaned.len();
        let reduction = if original_len > 0 {
            (original_len - cleaned_len) as f64 / original_len as f64 * 100.0
        } else {
            0.0
        };
        (original_len, cleaned_len, reduction)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let cleaner = DataCleaner::new(true, false);
        let data = vec![
            "apple".to_string(),
            "banana".to_string(),
            "apple".to_string(),
            "cherry".to_string(),
        ];
        let cleaned = cleaner.clean_dataset(data);
        assert_eq!(cleaned.len(), 3);
        assert!(cleaned.contains(&"apple".to_string()));
    }

    #[test]
    fn test_normalization() {
        let cleaner = DataCleaner::new(false, true);
        let data = vec!["  APPLE  ".to_string(), "Banana!".to_string()];
        let cleaned = cleaner.clean_dataset(data);
        assert_eq!(cleaned[0], "apple");
        assert_eq!(cleaned[1], "banana");
    }

    #[test]
    fn test_statistics() {
        let cleaner = DataCleaner::new(true, false);
        let original = vec![
            "apple".to_string(),
            "apple".to_string(),
            "banana".to_string(),
        ];
        let cleaned = cleaner.clean_dataset(original.clone());
        let stats = cleaner.statistics(&original, &cleaned);
        assert_eq!(stats.0, 3);
        assert_eq!(stats.1, 2);
        assert!((stats.2 - 33.33).abs() < 0.1);
    }
}