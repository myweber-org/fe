
use std::collections::HashSet;
use std::error::Error;

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

    pub fn clean_email(&self, email: &str) -> Result<String, Box<dyn Error>> {
        let cleaned = email.trim().to_lowercase();
        if !cleaned.contains('@') {
            return Err("Invalid email format".into());
        }
        Ok(cleaned)
    }

    pub fn get_unique_count(&self) -> usize {
        self.dedupe_set.len()
    }

    pub fn reset(&mut self) {
        self.dedupe_set.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalization() {
        let cleaner = DataCleaner::new();
        assert_eq!(cleaner.normalize_string("  TEST  "), "test");
    }

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        assert!(cleaner.deduplicate("hello"));
        assert!(!cleaner.deduplicate("  HELLO  "));
        assert_eq!(cleaner.get_unique_count(), 1);
    }

    #[test]
    fn test_email_cleaning() {
        let cleaner = DataCleaner::new();
        assert_eq!(cleaner.clean_email(" USER@EXAMPLE.COM ").unwrap(), "user@example.com");
        assert!(cleaner.clean_email("invalid").is_err());
    }
}