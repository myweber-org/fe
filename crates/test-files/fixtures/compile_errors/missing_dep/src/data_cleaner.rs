
use std::collections::HashSet;

pub struct DataCleaner {
    pub data: Vec<String>,
}

impl DataCleaner {
    pub fn new(data: Vec<String>) -> Self {
        DataCleaner { data }
    }

    pub fn remove_duplicates(&mut self) {
        let unique_set: HashSet<String> = self.data.drain(..).collect();
        self.data = unique_set.into_iter().collect();
    }

    pub fn normalize_strings(&mut self) {
        for item in &mut self.data {
            *item = item.trim().to_lowercase();
        }
    }

    pub fn clean(&mut self) -> &Vec<String> {
        self.remove_duplicates();
        self.normalize_strings();
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_cleaner() {
        let raw_data = vec![
            "  Apple  ".to_string(),
            "banana".to_string(),
            "  APPLE  ".to_string(),
            "Banana".to_string(),
            "cherry".to_string(),
        ];

        let mut cleaner = DataCleaner::new(raw_data);
        let cleaned = cleaner.clean();

        assert_eq!(cleaned.len(), 3);
        assert!(cleaned.contains(&"apple".to_string()));
        assert!(cleaned.contains(&"banana".to_string()));
        assert!(cleaned.contains(&"cherry".to_string()));
    }
}