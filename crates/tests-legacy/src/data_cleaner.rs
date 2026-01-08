use std::collections::HashSet;

pub struct DataCleaner {
    entries: Vec<String>,
}

impl DataCleaner {
    pub fn new(entries: Vec<String>) -> Self {
        DataCleaner { entries }
    }

    pub fn clean(&mut self) -> Vec<String> {
        let unique_set: HashSet<String> = self.entries.drain(..).collect();
        let mut unique_vec: Vec<String> = unique_set.into_iter().collect();
        unique_vec.sort();
        unique_vec
    }

    pub fn process_raw_data(raw_data: &[&str]) -> Vec<String> {
        let entries: Vec<String> = raw_data.iter().map(|s| s.to_string()).collect();
        let mut cleaner = DataCleaner::new(entries);
        cleaner.clean()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_duplicates() {
        let raw_data = vec!["apple", "orange", "banana", "apple", "orange"];
        let cleaned = DataCleaner::process_raw_data(&raw_data);
        assert_eq!(cleaned, vec!["apple", "banana", "orange"]);
    }

    #[test]
    fn test_empty_input() {
        let raw_data: Vec<&str> = vec![];
        let cleaned = DataCleaner::process_raw_data(&raw_data);
        assert!(cleaned.is_empty());
    }
}