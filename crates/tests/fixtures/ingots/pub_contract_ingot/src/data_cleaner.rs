
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

    pub fn clean_csv_row(&mut self, row: Vec<String>) -> Result<Vec<String>, Box<dyn Error>> {
        if row.len() < 2 {
            return Err("Row must have at least 2 columns".into());
        }

        let mut cleaned = Vec::new();
        for (i, field) in row.iter().enumerate() {
            if i == 0 {
                if !self.deduplicate(field) {
                    return Err(format!("Duplicate primary key: {}", field).into());
                }
                cleaned.push(field.clone());
            } else {
                cleaned.push(self.normalize_string(field));
            }
        }
        Ok(cleaned)
    }

    pub fn reset(&mut self) {
        self.dedupe_set.clear();
    }

    pub fn unique_count(&self) -> usize {
        self.dedupe_set.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalization() {
        let cleaner = DataCleaner::new();
        assert_eq!(cleaner.normalize_string("  TEST  "), "test");
        assert_eq!(cleaner.normalize_string("MiXeD CaSe"), "mixed case");
    }

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        assert!(cleaner.deduplicate("apple"));
        assert!(!cleaner.deduplicate("  APPLE  "));
        assert!(cleaner.deduplicate("banana"));
        assert_eq!(cleaner.unique_count(), 2);
    }

    #[test]
    fn test_csv_cleaning() {
        let mut cleaner = DataCleaner::new();
        let row = vec!["ID123".to_string(), "  John Doe  ".to_string(), "USA".to_string()];
        let cleaned = cleaner.clean_csv_row(row).unwrap();
        assert_eq!(cleaned, vec!["ID123", "john doe", "usa"]);
    }
}
use regex::Regex;

pub fn normalize_string(input: &str) -> String {
    let trimmed = input.trim();
    
    let re_multispace = Regex::new(r"\s+").unwrap();
    let single_spaced = re_multispace.replace_all(trimmed, " ");
    
    let re_non_alphanumeric = Regex::new(r"[^a-zA-Z0-9\s\-\.]").unwrap();
    let cleaned = re_non_alphanumeric.replace_all(&single_spaced, "");
    
    cleaned.to_lowercase()
}

pub fn extract_numeric(input: &str) -> Option<f64> {
    let re = Regex::new(r"-?\d+(\.\d+)?").unwrap();
    re.find(input)
        .and_then(|m| m.as_str().parse::<f64>().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_string() {
        assert_eq!(
            normalize_string("  Hello   World!!  TEST-123  "),
            "hello world test-123"
        );
        assert_eq!(
            normalize_string("Data@#$%Science   Project"),
            "data science project"
        );
    }

    #[test]
    fn test_extract_numeric() {
        assert_eq!(extract_numeric("Price: 29.99 USD"), Some(29.99));
        assert_eq!(extract_numeric("Temperature: -5.5°C"), Some(-5.5));
        assert_eq!(extract_numeric("No numbers here"), None);
    }
}