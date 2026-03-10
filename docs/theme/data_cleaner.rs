use std::collections::HashSet;
use std::error::Error;

pub struct DataCleaner {
    dedupe_set: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            dedupe_set: HashSet::new(),
        }
    }

    pub fn normalize_text(&self, text: &str) -> String {
        text.trim()
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect()
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

    pub fn clean_dataset(&mut self, data: Vec<&str>) -> Result<Vec<String>, Box<dyn Error>> {
        let mut cleaned = Vec::new();
        
        for item in data {
            if self.deduplicate(item) {
                cleaned.push(self.normalize_text(item));
            }
        }
        
        if cleaned.is_empty() {
            return Err("No unique items found after cleaning".into());
        }
        
        Ok(cleaned)
    }

    pub fn reset(&mut self) {
        self.dedupe_set.clear();
    }

    pub fn unique_count(&self) -> usize {
        self.dedupe_set.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalization() {
        let cleaner = DataCleaner::new();
        assert_eq!(cleaner.normalize_text("  HELLO World!  "), "hello world");
    }

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        assert!(cleaner.deduplicate("test"));
        assert!(!cleaner.deduplicate("TEST"));
        assert!(cleaner.deduplicate("different"));
    }

    #[test]
    fn test_clean_dataset() {
        let mut cleaner = DataCleaner::new();
        let data = vec!["apple", "APPLE", "banana", "  Banana  "];
        let result = cleaner.clean_dataset(data).unwrap();
        assert_eq!(result, vec!["apple", "banana"]);
        assert_eq!(cleaner.unique_count(), 2);
    }
}use std::collections::HashMap;

pub struct DataCleaner {
    threshold: f64,
}

impl DataCleaner {
    pub fn new(threshold: f64) -> Self {
        DataCleaner { threshold }
    }

    pub fn remove_outliers_iqr(&self, data: &[f64]) -> Vec<f64> {
        if data.len() < 4 {
            return data.to_vec();
        }

        let mut sorted_data = data.to_vec();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let q1 = Self::calculate_quartile(&sorted_data, 0.25);
        let q3 = Self::calculate_quartile(&sorted_data, 0.75);
        let iqr = q3 - q1;

        let lower_bound = q1 - self.threshold * iqr;
        let upper_bound = q3 + self.threshold * iqr;

        data.iter()
            .filter(|&&x| x >= lower_bound && x <= upper_bound)
            .copied()
            .collect()
    }

    fn calculate_quartile(sorted_data: &[f64], percentile: f64) -> f64 {
        let index = percentile * (sorted_data.len() - 1) as f64;
        let lower = index.floor() as usize;
        let upper = index.ceil() as usize;

        if lower == upper {
            sorted_data[lower]
        } else {
            let weight = index - lower as f64;
            sorted_data[lower] * (1.0 - weight) + sorted_data[upper] * weight
        }
    }

    pub fn analyze_dataset(&self, data: &[f64]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if !data.is_empty() {
            let sum: f64 = data.iter().sum();
            let mean = sum / data.len() as f64;
            let variance: f64 = data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / data.len() as f64;
            let std_dev = variance.sqrt();

            stats.insert("mean".to_string(), mean);
            stats.insert("std_dev".to_string(), std_dev);
            stats.insert("count".to_string(), data.len() as f64);
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_outliers() {
        let cleaner = DataCleaner::new(1.5);
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 100.0];
        let cleaned = cleaner.remove_outliers_iqr(&data);
        
        assert_eq!(cleaned.len(), 5);
        assert!(!cleaned.contains(&100.0));
    }

    #[test]
    fn test_analyze_dataset() {
        let cleaner = DataCleaner::new(1.5);
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = cleaner.analyze_dataset(&data);
        
        assert_eq!(stats["mean"], 3.0);
        assert_eq!(stats["count"], 5.0);
    }
}use std::collections::HashSet;

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
use std::collections::HashSet;

pub fn clean_dataset<T: Eq + std::hash::Hash + Clone>(
    data: &[T],
    invalid_items: &HashSet<T>,
) -> Vec<T> {
    data.iter()
        .filter(|item| !invalid_items.contains(item))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_dataset() {
        let data = vec![1, 2, 3, 4, 5];
        let invalid: HashSet<i32> = [2, 4].iter().cloned().collect();
        let cleaned = clean_dataset(&data, &invalid);
        assert_eq!(cleaned, vec![1, 3, 5]);
    }

    #[test]
    fn test_clean_dataset_empty_invalid() {
        let data = vec!["apple", "banana", "cherry"];
        let invalid: HashSet<&str> = HashSet::new();
        let cleaned = clean_dataset(&data, &invalid);
        assert_eq!(cleaned, data);
    }

    #[test]
    fn test_clean_dataset_all_invalid() {
        let data = vec![10.5, 20.3, 30.7];
        let invalid: HashSet<f64> = [10.5, 20.3, 30.7].iter().cloned().collect();
        let cleaned = clean_dataset(&data, &invalid);
        assert!(cleaned.is_empty());
    }
}