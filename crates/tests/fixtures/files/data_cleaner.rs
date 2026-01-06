use std::collections::HashSet;

pub struct DataCleaner {
    records: Vec<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            records: Vec::new(),
        }
    }

    pub fn add_record(&mut self, record: String) {
        self.records.push(record);
    }

    pub fn deduplicate(&mut self) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut unique_records = Vec::new();

        for record in self.records.drain(..) {
            if seen.insert(record.clone()) {
                unique_records.push(record);
            }
        }

        self.records = unique_records.clone();
        unique_records
    }

    pub fn normalize_whitespace(&mut self) {
        for record in &mut self.records {
            let normalized = record
                .split_whitespace()
                .collect::<Vec<&str>>()
                .join(" ");
            *record = normalized;
        }
    }

    pub fn to_lowercase(&mut self) {
        for record in &mut self.records {
            *record = record.to_lowercase();
        }
    }

    pub fn get_records(&self) -> &Vec<String> {
        &self.records
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("test".to_string());
        cleaner.add_record("test".to_string());
        cleaner.add_record("unique".to_string());

        let deduped = cleaner.deduplicate();
        assert_eq!(deduped.len(), 2);
        assert!(deduped.contains(&"test".to_string()));
        assert!(deduped.contains(&"unique".to_string()));
    }

    #[test]
    fn test_normalization() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("  multiple   spaces   ".to_string());
        cleaner.normalize_whitespace();

        assert_eq!(cleaner.get_records()[0], "multiple spaces");
    }
}
use std::collections::HashMap;

pub struct DataCleaner {
    pub null_values: Vec<String>,
    pub normalization_map: HashMap<String, String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        let mut normalization_map = HashMap::new();
        normalization_map.insert("USA".to_string(), "United States".to_string());
        normalization_map.insert("UK".to_string(), "United Kingdom".to_string());
        normalization_map.insert("UAE".to_string(), "United Arab Emirates".to_string());

        DataCleaner {
            null_values: vec!["null".to_string(), "NULL".to_string(), "".to_string(), "N/A".to_string()],
            normalization_map,
        }
    }

    pub fn clean_string(&self, input: &str) -> Option<String> {
        if self.null_values.contains(&input.to_string()) {
            return None;
        }

        let trimmed = input.trim();
        if trimmed.is_empty() {
            return None;
        }

        match self.normalization_map.get(trimmed) {
            Some(normalized) => Some(normalized.clone()),
            None => Some(trimmed.to_string()),
        }
    }

    pub fn clean_vector(&self, data: Vec<&str>) -> Vec<String> {
        data.iter()
            .filter_map(|&item| self.clean_string(item))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_string() {
        let cleaner = DataCleaner::new();
        
        assert_eq!(cleaner.clean_string("USA"), Some("United States".to_string()));
        assert_eq!(cleaner.clean_string("null"), None);
        assert_eq!(cleaner.clean_string("   "), None);
        assert_eq!(cleaner.clean_string("valid data"), Some("valid data".to_string()));
    }

    #[test]
    fn test_clean_vector() {
        let cleaner = DataCleaner::new();
        let data = vec!["USA", "null", "valid", "", "UK"];
        let cleaned = cleaner.clean_vector(data);
        
        assert_eq!(cleaned, vec!["United States", "valid", "United Kingdom"]);
    }
}