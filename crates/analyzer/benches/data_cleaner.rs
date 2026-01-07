
use std::collections::HashMap;

pub struct DataCleaner {
    pub null_values: Vec<String>,
    pub normalization_map: HashMap<String, String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        let mut normalization_map = HashMap::new();
        normalization_map.insert("N/A".to_string(), "".to_string());
        normalization_map.insert("NULL".to_string(), "".to_string());
        normalization_map.insert("NaN".to_string(), "".to_string());

        DataCleaner {
            null_values: vec![
                "".to_string(),
                "null".to_string(),
                "NULL".to_string(),
                "N/A".to_string(),
                "NaN".to_string(),
            ],
            normalization_map,
        }
    }

    pub fn clean_string(&self, input: &str) -> Option<String> {
        let trimmed = input.trim();
        
        if self.null_values.contains(&trimmed.to_string()) {
            return None;
        }

        if let Some(normalized) = self.normalization_map.get(trimmed) {
            return Some(normalized.clone());
        }

        Some(trimmed.to_string())
    }

    pub fn clean_vector(&self, data: Vec<String>) -> Vec<Option<String>> {
        data.iter()
            .map(|item| self.clean_string(item))
            .collect()
    }

    pub fn remove_nulls(&self, data: Vec<Option<String>>) -> Vec<String> {
        data.into_iter()
            .filter_map(|x| x)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_string() {
        let cleaner = DataCleaner::new();
        
        assert_eq!(cleaner.clean_string("  hello  "), Some("hello".to_string()));
        assert_eq!(cleaner.clean_string(""), None);
        assert_eq!(cleaner.clean_string("null"), None);
        assert_eq!(cleaner.clean_string("N/A"), Some("".to_string()));
        assert_eq!(cleaner.clean_string("NULL"), Some("".to_string()));
    }

    #[test]
    fn test_clean_vector() {
        let cleaner = DataCleaner::new();
        let data = vec![
            "  hello  ".to_string(),
            "".to_string(),
            "N/A".to_string(),
            "world".to_string(),
        ];
        
        let cleaned = cleaner.clean_vector(data);
        assert_eq!(cleaned.len(), 4);
        assert_eq!(cleaned[0], Some("hello".to_string()));
        assert_eq!(cleaned[1], None);
        assert_eq!(cleaned[2], Some("".to_string()));
        assert_eq!(cleaned[3], Some("world".to_string()));
    }

    #[test]
    fn test_remove_nulls() {
        let cleaner = DataCleaner::new();
        let data = vec![
            Some("hello".to_string()),
            None,
            Some("world".to_string()),
            None,
        ];
        
        let result = cleaner.remove_nulls(data);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "hello");
        assert_eq!(result[1], "world");
    }
}