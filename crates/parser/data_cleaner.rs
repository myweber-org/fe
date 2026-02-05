use std::collections::HashSet;

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

    pub fn clean_data(&mut self, data: Vec<&str>) -> Vec<String> {
        let mut cleaned = Vec::new();
        
        for item in data {
            if self.deduplicate(item) {
                cleaned.push(self.normalize_string(item));
            }
        }
        
        cleaned
    }

    pub fn get_unique_count(&self) -> usize {
        self.dedupe_set.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalization() {
        let cleaner = DataCleaner::new();
        assert_eq!(cleaner.normalize_string("  HELLO World  "), "hello world");
    }

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        assert!(cleaner.deduplicate("test"));
        assert!(!cleaner.deduplicate("TEST"));
        assert!(cleaner.deduplicate("another"));
    }

    #[test]
    fn test_clean_data() {
        let mut cleaner = DataCleaner::new();
        let data = vec!["Apple", "  apple ", "BANANA", "banana", "Cherry"];
        let cleaned = cleaner.clean_data(data);
        
        assert_eq!(cleaned.len(), 3);
        assert_eq!(cleaner.get_unique_count(), 3);
    }
}
use std::collections::HashMap;

pub struct DataCleaner {
    data: HashMap<String, Vec<Option<String>>>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            data: HashMap::new(),
        }
    }

    pub fn add_column(&mut self, name: &str, values: Vec<Option<String>>) {
        self.data.insert(name.to_string(), values);
    }

    pub fn clean_data(&mut self) -> HashMap<String, Vec<String>> {
        let mut cleaned = HashMap::new();
        
        for (col_name, values) in &self.data {
            let cleaned_values: Vec<String> = values
                .iter()
                .filter_map(|val| {
                    val.as_ref().map(|s| {
                        let trimmed = s.trim().to_string();
                        if trimmed.is_empty() {
                            None
                        } else {
                            Some(trimmed)
                        }
                    }).flatten()
                })
                .collect();
            
            if !cleaned_values.is_empty() {
                cleaned.insert(col_name.clone(), cleaned_values);
            }
        }
        
        cleaned
    }

    pub fn count_valid_entries(&self) -> usize {
        self.data.values()
            .flat_map(|values| values.iter())
            .filter(|val| {
                val.as_ref()
                    .map(|s| !s.trim().is_empty())
                    .unwrap_or(false)
            })
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_data() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_column("names", vec![
            Some("  John  ".to_string()),
            Some("".to_string()),
            None,
            Some("Alice".to_string()),
        ]);
        
        let cleaned = cleaner.clean_data();
        assert_eq!(cleaned.get("names").unwrap(), &vec!["John", "Alice"]);
        assert_eq!(cleaner.count_valid_entries(), 2);
    }
}use std::collections::HashSet;
use std::io::{self, BufRead};

pub fn clean_data(input: Vec<String>) -> Vec<String> {
    let mut unique_items: HashSet<String> = HashSet::new();
    for item in input {
        unique_items.insert(item.trim().to_string());
    }
    
    let mut sorted_items: Vec<String> = unique_items.into_iter().collect();
    sorted_items.sort();
    sorted_items
}

pub fn read_lines_from_stdin() -> Vec<String> {
    let stdin = io::stdin();
    let lines: Vec<String> = stdin.lock().lines()
        .filter_map(Result::ok)
        .collect();
    lines
}

pub fn process_input() -> Vec<String> {
    let raw_data = read_lines_from_stdin();
    clean_data(raw_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_data() {
        let input = vec![
            "banana".to_string(),
            "apple".to_string(),
            "banana".to_string(),
            "cherry".to_string(),
            "apple".to_string(),
        ];
        let result = clean_data(input);
        assert_eq!(result, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn test_empty_input() {
        let input: Vec<String> = vec![];
        let result = clean_data(input);
        assert!(result.is_empty());
    }
}