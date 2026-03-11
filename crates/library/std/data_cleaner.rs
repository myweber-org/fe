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
use regex::Regex;
use std::collections::HashSet;

pub fn clean_and_normalize(input: &str) -> String {
    let trimmed = input.trim();
    
    let re_multispace = Regex::new(r"\s+").unwrap();
    let normalized_spaces = re_multispace.replace_all(trimmed, " ");
    
    let re_special = Regex::new(r"[^\w\s\-\.]").unwrap();
    let cleaned = re_special.replace_all(&normalized_spaces, "");
    
    cleaned.to_lowercase()
}

pub fn deduplicate_words(text: &str) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    
    for word in words {
        if seen.insert(word) {
            result.push(word);
        }
    }
    
    result.join(" ")
}

pub fn validate_email(email: &str) -> bool {
    let re = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    re.is_match(email)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_and_normalize() {
        assert_eq!(clean_and_normalize("  Hello   WORLD!!  "), "hello world");
        assert_eq!(clean_and_normalize("Data@Process#2024"), "dataprocess2024");
    }

    #[test]
    fn test_deduplicate_words() {
        assert_eq!(deduplicate_words("hello world hello again"), "hello world again");
        assert_eq!(deduplicate_words("a b c a b"), "a b c");
    }

    #[test]
    fn test_validate_email() {
        assert!(validate_email("test@example.com"));
        assert!(!validate_email("invalid-email"));
        assert!(!validate_email("user@.com"));
    }
}