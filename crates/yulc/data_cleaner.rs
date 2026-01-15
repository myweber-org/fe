
use std::collections::HashSet;
use std::error::Error;

pub struct DataCleaner {
    records: Vec<String>,
    seen_ids: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            records: Vec::new(),
            seen_ids: HashSet::new(),
        }
    }

    pub fn add_record(&mut self, id: &str, data: &str) -> Result<(), Box<dyn Error>> {
        if id.trim().is_empty() {
            return Err("ID cannot be empty".into());
        }

        if data.trim().is_empty() {
            return Err("Data cannot be empty".into());
        }

        if !self.seen_ids.insert(id.to_string()) {
            return Err(format!("Duplicate ID found: {}", id).into());
        }

        let cleaned_data = data.trim().to_string();
        let record = format!("{}:{}", id, cleaned_data);
        self.records.push(record);

        Ok(())
    }

    pub fn get_clean_records(&self) -> &Vec<String> {
        &self.records
    }

    pub fn validate_email(email: &str) -> bool {
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return false;
        }

        let domain_parts: Vec<&str> = parts[1].split('.').collect();
        domain_parts.len() >= 2
            && !parts[0].is_empty()
            && !domain_parts.iter().any(|part| part.is_empty())
    }

    pub fn remove_whitespace(input: &str) -> String {
        input.chars().filter(|c| !c.is_whitespace()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_record() {
        let mut cleaner = DataCleaner::new();
        assert!(cleaner.add_record("001", "John Doe").is_ok());
        assert_eq!(cleaner.get_clean_records().len(), 1);
    }

    #[test]
    fn test_duplicate_id() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("001", "Data1").unwrap();
        assert!(cleaner.add_record("001", "Data2").is_err());
    }

    #[test]
    fn test_email_validation() {
        assert!(DataCleaner::validate_email("test@example.com"));
        assert!(!DataCleaner::validate_email("invalid-email"));
        assert!(!DataCleaner::validate_email("@domain.com"));
    }

    #[test]
    fn test_whitespace_removal() {
        let result = DataCleaner::remove_whitespace("  hello  world  ");
        assert_eq!(result, "helloworld");
    }
}