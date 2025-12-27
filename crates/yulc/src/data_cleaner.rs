
use std::collections::HashSet;

pub struct DataCleaner {
    pub remove_duplicates: bool,
    pub normalize_whitespace: bool,
    pub trim_strings: bool,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            remove_duplicates: true,
            normalize_whitespace: true,
            trim_strings: true,
        }
    }

    pub fn clean_data(&self, data: Vec<String>) -> Vec<String> {
        let mut processed_data = data;

        if self.trim_strings {
            processed_data = processed_data
                .into_iter()
                .map(|s| s.trim().to_string())
                .collect();
        }

        if self.normalize_whitespace {
            processed_data = processed_data
                .into_iter()
                .map(|s| s.split_whitespace().collect::<Vec<&str>>().join(" "))
                .collect();
        }

        if self.remove_duplicates {
            let unique_set: HashSet<String> = processed_data.into_iter().collect();
            processed_data = unique_set.into_iter().collect();
        }

        processed_data.sort();
        processed_data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_data() {
        let cleaner = DataCleaner::new();
        let input = vec![
            "  hello  world  ".to_string(),
            "hello world".to_string(),
            "foo   bar".to_string(),
            "foo bar".to_string(),
        ];

        let result = cleaner.clean_data(input);
        assert_eq!(result, vec!["foo bar", "hello world"]);
    }

    #[test]
    fn test_without_duplicate_removal() {
        let mut cleaner = DataCleaner::new();
        cleaner.remove_duplicates = false;
        
        let input = vec![
            "hello".to_string(),
            "hello".to_string(),
        ];

        let result = cleaner.clean_data(input);
        assert_eq!(result, vec!["hello", "hello"]);
    }
}