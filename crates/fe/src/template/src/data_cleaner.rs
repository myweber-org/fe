use std::collections::HashSet;

pub struct DataCleaner {
    pub remove_duplicates: bool,
    pub normalize_case: bool,
}

impl DataCleaner {
    pub fn new(remove_duplicates: bool, normalize_case: bool) -> Self {
        DataCleaner {
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

    pub fn validate_email(email: &str) -> bool {
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
            "Apple".to_string(),
            "banana".to_string(),
            "Apple".to_string(),
            "Cherry".to_string(),
        ];
        
        let result = cleaner.clean(data);
        assert_eq!(result.len(), 3);
        assert_eq!(result, vec!["Apple", "Cherry", "banana"]);
    }

    #[test]
    fn test_clean_with_normalization() {
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
    fn test_email_validation() {
        assert!(DataCleaner::validate_email("test@example.com"));
        assert!(!DataCleaner::validate_email("invalid-email"));
        assert!(!DataCleaner::validate_email("@domain.com"));
        assert!(!DataCleaner::validate_email("user@.com"));
    }
}use std::collections::HashSet;

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
            .map(|entry| entry.trim().to_lowercase())
            .collect()
    }
}

pub fn validate_email_format(email: &str) -> bool {
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return false;
    }

    let domain_parts: Vec<&str> = parts[1].split('.').collect();
    domain_parts.len() >= 2 && !parts[0].is_empty()
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
    }

    #[test]
    fn test_normalization() {
        let cleaner = DataCleaner::new(false, true);
        let data = vec![
            "  APPLE  ".to_string(),
            "Banana".to_string(),
            "  CHERRY  ".to_string(),
        ];
        let cleaned = cleaner.clean_dataset(data);
        assert_eq!(cleaned[0], "apple");
        assert_eq!(cleaned[1], "banana");
        assert_eq!(cleaned[2], "cherry");
    }

    #[test]
    fn test_email_validation() {
        assert!(validate_email_format("user@example.com"));
        assert!(!validate_email_format("invalid-email"));
        assert!(!validate_email_format("@domain.com"));
    }
}