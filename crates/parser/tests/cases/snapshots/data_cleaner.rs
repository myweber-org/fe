use std::collections::HashSet;

pub struct DataCleaner {
    deduplication_cache: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            deduplication_cache: HashSet::new(),
        }
    }

    pub fn normalize_text(&self, input: &str) -> String {
        input.trim().to_lowercase()
    }

    pub fn remove_duplicates(&mut self, items: Vec<String>) -> Vec<String> {
        let mut unique_items = Vec::new();
        
        for item in items {
            let normalized = self.normalize_text(&item);
            if self.deduplication_cache.insert(normalized.clone()) {
                unique_items.push(item);
            }
        }
        
        unique_items
    }

    pub fn validate_email(&self, email: &str) -> bool {
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return false;
        }
        
        let domain_parts: Vec<&str> = parts[1].split('.').collect();
        domain_parts.len() >= 2 && !parts[0].is_empty()
    }

    pub fn clean_whitespace(&self, input: &str) -> String {
        input.split_whitespace().collect::<Vec<&str>>().join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_text() {
        let cleaner = DataCleaner::new();
        assert_eq!(cleaner.normalize_text("  HELLO World  "), "hello world");
    }

    #[test]
    fn test_remove_duplicates() {
        let mut cleaner = DataCleaner::new();
        let data = vec![
            "test@example.com".to_string(),
            "TEST@example.com".to_string(),
            "another@test.com".to_string(),
        ];
        
        let result = cleaner.remove_duplicates(data);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_validate_email() {
        let cleaner = DataCleaner::new();
        assert!(cleaner.validate_email("user@example.com"));
        assert!(!cleaner.validate_email("invalid-email"));
    }
}