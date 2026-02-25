use std::collections::HashSet;

pub fn clean_data<T: Eq + std::hash::Hash + Clone>(data: &[T]) -> Vec<T> {
    let mut seen = HashSet::new();
    data.iter()
        .filter(|item| {
            let is_unique = seen.insert(*item);
            is_unique
        })
        .cloned()
        .collect()
}

pub fn remove_nulls<T: Default + PartialEq>(data: &[Option<T>]) -> Vec<T> {
    data.iter()
        .filter_map(|item| {
            if let Some(value) = item {
                if *value != T::default() {
                    return Some(value.clone());
                }
            }
            None
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_data_removes_duplicates() {
        let data = vec![1, 2, 2, 3, 4, 4, 5];
        let cleaned = clean_data(&data);
        assert_eq!(cleaned, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_remove_nulls_filters_none_and_default() {
        let data = vec![Some(1), None, Some(0), Some(2), Some(0)];
        let cleaned = remove_nulls(&data);
        assert_eq!(cleaned, vec![1, 2]);
    }
}
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

    pub fn validate_records(&self) -> Result<(), String> {
        for (index, record) in self.records.iter().enumerate() {
            if record.trim().is_empty() {
                return Err(format!("Empty record found at index {}", index));
            }
            
            if record.len() > 1000 {
                return Err(format!("Record too long at index {}", index));
            }
        }
        Ok(())
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
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

        let unique = cleaner.deduplicate();
        assert_eq!(unique.len(), 2);
        assert_eq!(cleaner.get_record_count(), 2);
    }

    #[test]
    fn test_validation() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("valid record".to_string());
        
        assert!(cleaner.validate_records().is_ok());
        
        cleaner.add_record("".to_string());
        assert!(cleaner.validate_records().is_err());
    }
}