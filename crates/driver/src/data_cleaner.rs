
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
                .into_iter()
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

    pub fn clean_with_callback<F>(&self, data: Vec<String>, mut callback: F) -> Vec<String>
    where
        F: FnMut(&str),
    {
        let cleaned = self.clean(data);
        for item in &cleaned {
            callback(item);
        }
        cleaned
    }
}

pub fn validate_email(email: &str) -> bool {
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return false;
    }
    
    let domain_parts: Vec<&str> = parts[1].split('.').collect();
    domain_parts.len() >= 2 && !email.contains(' ')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_cleaner() {
        let cleaner = DataCleaner::new(true, true);
        let data = vec![
            "Apple".to_string(),
            "banana".to_string(),
            "Apple".to_string(),
            "Cherry".to_string(),
        ];
        
        let result = cleaner.clean(data);
        assert_eq!(result, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn test_email_validation() {
        assert!(validate_email("test@example.com"));
        assert!(!validate_email("invalid-email"));
        assert!(!validate_email("test@com"));
    }
}
use std::collections::HashSet;

pub struct DataCleaner {
    records: Vec<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            records: Vec::new(),
        }
    }

    pub fn add_record(&mut self, record: String) {
        self.records.push(record);
    }

    pub fn deduplicate(&mut self) {
        let mut seen = HashSet::new();
        self.records.retain(|record| seen.insert(record.clone()));
    }

    pub fn normalize_whitespace(&mut self) {
        for record in self.records.iter_mut() {
            let normalized = record
                .split_whitespace()
                .collect::<Vec<&str>>()
                .join(" ");
            *record = normalized;
        }
    }

    pub fn to_lowercase(&mut self) {
        for record in self.records.iter_mut() {
            *record = record.to_lowercase();
        }
    }

    pub fn get_records(&self) -> &Vec<String> {
        &self.records
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

pub fn process_string(input: &str) -> String {
    input
        .trim()
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("test".to_string());
        cleaner.add_record("test".to_string());
        cleaner.add_record("another".to_string());
        
        cleaner.deduplicate();
        
        assert_eq!(cleaner.get_records().len(), 2);
    }

    #[test]
    fn test_normalization() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("  Multiple   Spaces   Here  ".to_string());
        
        cleaner.normalize_whitespace();
        
        assert_eq!(cleaner.get_records()[0], "Multiple Spaces Here");
    }

    #[test]
    fn test_process_string() {
        let result = process_string("  HELLO    World  ");
        assert_eq!(result, "hello world");
    }
}