use std::collections::HashSet;

pub struct DataCleaner {
    dedupe_set: HashSet<String>,
    normalize_case: bool,
}

impl DataCleaner {
    pub fn new(normalize_case: bool) -> Self {
        DataCleaner {
            dedupe_set: HashSet::new(),
            normalize_case,
        }
    }

    pub fn process(&mut self, input: &str) -> Option<String> {
        let processed = if self.normalize_case {
            input.to_lowercase()
        } else {
            input.to_string()
        };

        let trimmed = processed.trim().to_string();

        if trimmed.is_empty() {
            return None;
        }

        if self.dedupe_set.contains(&trimmed) {
            return None;
        }

        self.dedupe_set.insert(trimmed.clone());
        Some(trimmed)
    }

    pub fn reset(&mut self) {
        self.dedupe_set.clear();
    }

    pub fn processed_count(&self) -> usize {
        self.dedupe_set.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new(false);
        assert_eq!(cleaner.process("hello"), Some("hello".to_string()));
        assert_eq!(cleaner.process("hello"), None);
        assert_eq!(cleaner.process("world"), Some("world".to_string()));
        assert_eq!(cleaner.processed_count(), 2);
    }

    #[test]
    fn test_case_normalization() {
        let mut cleaner = DataCleaner::new(true);
        assert_eq!(cleaner.process("HELLO"), Some("hello".to_string()));
        assert_eq!(cleaner.process("Hello"), None);
        assert_eq!(cleaner.process("WORLD"), Some("world".to_string()));
    }

    #[test]
    fn test_whitespace_trimming() {
        let mut cleaner = DataCleaner::new(false);
        assert_eq!(cleaner.process("  test  "), Some("test".to_string()));
        assert_eq!(cleaner.process("test"), None);
    }

    #[test]
    fn test_empty_input() {
        let mut cleaner = DataCleaner::new(false);
        assert_eq!(cleaner.process(""), None);
        assert_eq!(cleaner.process("   "), None);
    }
}