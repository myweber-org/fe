use std::collections::HashSet;
use std::hash::Hash;

pub fn deduplicate<T: Eq + Hash + Clone>(items: Vec<T>) -> Vec<T> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    
    for item in items {
        if seen.insert(item.clone()) {
            result.push(item);
        }
    }
    result
}

pub fn normalize_strings(strings: Vec<String>) -> Vec<String> {
    strings
        .into_iter()
        .map(|s| s.trim().to_lowercase())
        .collect()
}

pub fn filter_empty(strings: Vec<String>) -> Vec<String> {
    strings
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn clean_data(strings: Vec<String>) -> Vec<String> {
    let normalized = normalize_strings(strings);
    let filtered = filter_empty(normalized);
    deduplicate(filtered)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate() {
        let input = vec![1, 2, 2, 3, 4, 4, 5];
        let result = deduplicate(input);
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_normalize_strings() {
        let input = vec!["  HELLO  ".to_string(), "World".to_string()];
        let result = normalize_strings(input);
        assert_eq!(result, vec!["hello", "world"]);
    }

    #[test]
    fn test_filter_empty() {
        let input = vec!["hello".to_string(), "".to_string(), "world".to_string()];
        let result = filter_empty(input);
        assert_eq!(result, vec!["hello", "world"]);
    }

    #[test]
    fn test_clean_data() {
        let input = vec![
            "  Apple  ".to_string(),
            "apple".to_string(),
            "".to_string(),
            "Banana  ".to_string(),
            "banana".to_string(),
        ];
        let result = clean_data(input);
        assert_eq!(result, vec!["apple", "banana"]);
    }
}use std::collections::HashSet;
use std::error::Error;

pub struct DataCleaner {
    records: Vec<String>,
    seen: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            records: Vec::new(),
            seen: HashSet::new(),
        }
    }

    pub fn add_record(&mut self, record: &str) -> Result<(), Box<dyn Error>> {
        let trimmed = record.trim();
        
        if trimmed.is_empty() {
            return Err("Empty record not allowed".into());
        }

        if trimmed.len() > 1000 {
            return Err("Record exceeds maximum length".into());
        }

        if self.seen.contains(trimmed) {
            return Err("Duplicate record detected".into());
        }

        self.seen.insert(trimmed.to_string());
        self.records.push(trimmed.to_string());
        Ok(())
    }

    pub fn get_clean_records(&self) -> &Vec<String> {
        &self.records
    }

    pub fn validate_all(&self) -> bool {
        !self.records.is_empty() && self.records.len() == self.seen.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("test").unwrap();
        assert!(cleaner.add_record("test").is_err());
    }

    #[test]
    fn test_validation() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("valid").unwrap();
        assert!(cleaner.validate_all());
    }
}