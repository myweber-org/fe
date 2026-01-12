use std::collections::HashMap;

pub struct DataCleaner {
    filters: Vec<Box<dyn Fn(&HashMap<String, String>) -> bool>>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            filters: Vec::new(),
        }
    }

    pub fn add_filter<F>(&mut self, filter: F)
    where
        F: Fn(&HashMap<String, String>) -> bool + 'static,
    {
        self.filters.push(Box::new(filter));
    }

    pub fn clean(&self, data: Vec<HashMap<String, String>>) -> Vec<HashMap<String, String>> {
        data.into_iter()
            .filter(|entry| self.filters.iter().all(|f| f(entry)))
            .collect()
    }
}

pub fn create_default_cleaner() -> DataCleaner {
    let mut cleaner = DataCleaner::new();
    
    cleaner.add_filter(|entry| {
        entry.contains_key("id") && !entry.get("id").unwrap().is_empty()
    });
    
    cleaner.add_filter(|entry| {
        entry.get("timestamp")
            .and_then(|ts| ts.parse::<u64>().ok())
            .map_or(false, |ts| ts > 0)
    });
    
    cleaner
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_cleaner() {
        let cleaner = create_default_cleaner();
        
        let mut valid_entry = HashMap::new();
        valid_entry.insert("id".to_string(), "123".to_string());
        valid_entry.insert("timestamp".to_string(), "1672531200".to_string());
        
        let mut invalid_entry = HashMap::new();
        invalid_entry.insert("id".to_string(), "".to_string());
        invalid_entry.insert("timestamp".to_string(), "0".to_string());
        
        let data = vec![valid_entry, invalid_entry];
        let cleaned = cleaner.clean(data);
        
        assert_eq!(cleaned.len(), 1);
        assert_eq!(cleaned[0].get("id").unwrap(), "123");
    }
}use std::collections::HashSet;

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
        data.into_iter()
            .filter(|item| self.deduplicate(item))
            .map(|item| self.normalize_text(item))
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
        assert_eq!(cleaner.normalize_text("TEST123"), "test123");
    }
}