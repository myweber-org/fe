use std::collections::HashSet;

pub struct DataCleaner {
    data: Vec<String>,
}

impl DataCleaner {
    pub fn new(data: Vec<String>) -> Self {
        DataCleaner { data }
    }

    pub fn deduplicate(&mut self) -> &mut Self {
        let mut seen = HashSet::new();
        self.data.retain(|item| seen.insert(item.clone()));
        self
    }

    pub fn validate_length(&self, min_len: usize, max_len: usize) -> Vec<&String> {
        self.data
            .iter()
            .filter(|item| item.len() >= min_len && item.len() <= max_len)
            .collect()
    }

    pub fn trim_whitespace(&mut self) -> &mut Self {
        for item in &mut self.data {
            *item = item.trim().to_string();
        }
        self
    }

    pub fn get_data(&self) -> &Vec<String> {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate() {
        let mut cleaner = DataCleaner::new(vec![
            "apple".to_string(),
            "banana".to_string(),
            "apple".to_string(),
            "cherry".to_string(),
        ]);
        
        cleaner.deduplicate();
        let result = cleaner.get_data();
        
        assert_eq!(result.len(), 3);
        assert!(result.contains(&"apple".to_string()));
        assert!(result.contains(&"banana".to_string()));
        assert!(result.contains(&"cherry".to_string()));
    }

    #[test]
    fn test_validate_length() {
        let cleaner = DataCleaner::new(vec![
            "cat".to_string(),
            "elephant".to_string(),
            "dog".to_string(),
            "mouse".to_string(),
        ]);
        
        let valid = cleaner.validate_length(3, 5);
        assert_eq!(valid.len(), 2);
    }

    #[test]
    fn test_trim_whitespace() {
        let mut cleaner = DataCleaner::new(vec![
            "  hello  ".to_string(),
            "world\n".to_string(),
            "\tdata\t".to_string(),
        ]);
        
        cleaner.trim_whitespace();
        let result = cleaner.get_data();
        
        assert_eq!(result[0], "hello");
        assert_eq!(result[1], "world");
        assert_eq!(result[2], "data");
    }
}