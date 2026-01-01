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
        let mut processed = data;

        if self.normalize_case {
            processed = processed
                .into_iter()
                .map(|s| s.to_lowercase())
                .collect();
        }

        if self.remove_duplicates {
            let unique_set: HashSet<String> = processed.into_iter().collect();
            processed = unique_set.into_iter().collect();
        }

        processed.sort();
        processed
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleaner_removes_duplicates() {
        let cleaner = DataCleaner::new(true, false);
        let input = vec![
            "apple".to_string(),
            "banana".to_string(),
            "apple".to_string(),
            "cherry".to_string(),
        ];
        
        let result = cleaner.clean(input);
        assert_eq!(result.len(), 3);
        assert_eq!(result, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn test_cleaner_normalizes_case() {
        let cleaner = DataCleaner::new(false, true);
        let input = vec![
            "Apple".to_string(),
            "BANANA".to_string(),
            "Cherry".to_string(),
        ];
        
        let result = cleaner.clean(input);
        assert!(result.iter().all(|s| s.chars().all(|c| c.is_lowercase())));
    }
}