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

    pub fn normalize_text(&self, input: &str) -> String {
        input.trim().to_lowercase()
    }

    pub fn deduplicate(&mut self, item: &str) -> bool {
        let normalized = self.normalize_text(item);
        if self.dedupe_set.contains(&normalized) {
            false
        } else {
            self.dedupe_set.insert(normalized);
            true
        }
    }

    pub fn clean_dataset(&mut self, data: Vec<&str>) -> Vec<String> {
        data.iter()
            .filter(|&&item| self.deduplicate(item))
            .map(|&item| self.normalize_text(item))
            .collect()
    }

    pub fn get_unique_count(&self) -> usize {
        self.dedupe_set.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        let data = vec!["Apple", "apple", "APPLE", "Banana", "banana"];
        let cleaned = cleaner.clean_dataset(data);
        
        assert_eq!(cleaned.len(), 2);
        assert_eq!(cleaner.get_unique_count(), 2);
        assert!(cleaned.contains(&"apple".to_string()));
        assert!(cleaned.contains(&"banana".to_string()));
    }

    #[test]
    fn test_normalization() {
        let cleaner = DataCleaner::new();
        assert_eq!(cleaner.normalize_text("  HELLO World  "), "hello world");
    }
}use std::collections::HashSet;

pub struct DataCleaner {
    pub remove_duplicates: bool,
    pub normalize_case: bool,
}

impl DataCleaner {
    pub fn new(remove_duplicates: bool, normalize_case: bool) -> Self {
        Self {
            remove_duplicates,
            normalize_case,
        }
    }

    pub fn clean(&self, data: Vec<String>) -> Vec<String> {
        let mut processed_data = data;

        if self.normalize_case {
            processed_data = processed_data
                .iter()
                .map(|s| s.to_lowercase())
                .collect();
        }

        if self.remove_duplicates {
            let unique_set: HashSet<String> = processed_data.into_iter().collect();
            processed_data = unique_set.into_iter().collect();
        }

        processed_data.sort();
        processed_data
    }

    pub fn validate_email(&self, email: &str) -> bool {
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return false;
        }
        
        let domain_parts: Vec<&str> = parts[1].split('.').collect();
        domain_parts.len() >= 2 && 
        !parts[0].is_empty() && 
        !domain_parts.iter().any(|part| part.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_with_duplicates() {
        let cleaner = DataCleaner::new(true, false);
        let data = vec![
            "apple".to_string(),
            "banana".to_string(),
            "apple".to_string(),
            "cherry".to_string(),
        ];
        
        let result = cleaner.clean(data);
        assert_eq!(result, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn test_clean_with_case_normalization() {
        let cleaner = DataCleaner::new(false, true);
        let data = vec![
            "Apple".to_string(),
            "BANANA".to_string(),
            "cherry".to_string(),
        ];
        
        let result = cleaner.clean(data);
        assert_eq!(result, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn test_validate_email() {
        let cleaner = DataCleaner::new(false, false);
        
        assert!(cleaner.validate_email("test@example.com"));
        assert!(cleaner.validate_email("user.name@domain.co.uk"));
        assert!(!cleaner.validate_email("invalid-email"));
        assert!(!cleaner.validate_email("@domain.com"));
        assert!(!cleaner.validate_email("user@.com"));
    }
}