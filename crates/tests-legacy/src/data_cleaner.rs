use std::collections::HashSet;

pub struct DataCleaner {
    entries: Vec<String>,
}

impl DataCleaner {
    pub fn new(entries: Vec<String>) -> Self {
        DataCleaner { entries }
    }

    pub fn clean(&mut self) -> Vec<String> {
        let unique_set: HashSet<String> = self.entries.drain(..).collect();
        let mut unique_vec: Vec<String> = unique_set.into_iter().collect();
        unique_vec.sort();
        unique_vec
    }

    pub fn process_raw_data(raw_data: &[&str]) -> Vec<String> {
        let entries: Vec<String> = raw_data.iter().map(|s| s.to_string()).collect();
        let mut cleaner = DataCleaner::new(entries);
        cleaner.clean()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_duplicates() {
        let raw_data = vec!["apple", "orange", "banana", "apple", "orange"];
        let cleaned = DataCleaner::process_raw_data(&raw_data);
        assert_eq!(cleaned, vec!["apple", "banana", "orange"]);
    }

    #[test]
    fn test_empty_input() {
        let raw_data: Vec<&str> = vec![];
        let cleaned = DataCleaner::process_raw_data(&raw_data);
        assert!(cleaned.is_empty());
    }
}use std::collections::HashMap;

pub struct DataCleaner {
    data: Vec<f64>,
}

impl DataCleaner {
    pub fn new(data: Vec<f64>) -> Self {
        DataCleaner { data }
    }

    pub fn remove_outliers_iqr(&mut self) -> Vec<f64> {
        if self.data.len() < 4 {
            return self.data.clone();
        }

        let mut sorted_data = self.data.clone();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let q1_index = (sorted_data.len() as f64 * 0.25).floor() as usize;
        let q3_index = (sorted_data.len() as f64 * 0.75).floor() as usize;

        let q1 = sorted_data[q1_index];
        let q3 = sorted_data[q3_index];
        let iqr = q3 - q1;

        let lower_bound = q1 - 1.5 * iqr;
        let upper_bound = q3 + 1.5 * iqr;

        self.data
            .iter()
            .filter(|&&x| x >= lower_bound && x <= upper_bound)
            .cloned()
            .collect()
    }

    pub fn get_summary_stats(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        if self.data.is_empty() {
            return stats;
        }

        let sum: f64 = self.data.iter().sum();
        let mean = sum / self.data.len() as f64;

        let variance: f64 = self.data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / self.data.len() as f64;
        let std_dev = variance.sqrt();

        let mut sorted = self.data.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median = if sorted.len() % 2 == 0 {
            (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
        } else {
            sorted[sorted.len() / 2]
        };

        stats.insert("mean".to_string(), mean);
        stats.insert("median".to_string(), median);
        stats.insert("std_dev".to_string(), std_dev);
        stats.insert("min".to_string(), *sorted.first().unwrap());
        stats.insert("max".to_string(), *sorted.last().unwrap());

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outlier_removal() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 100.0];
        let mut cleaner = DataCleaner::new(data);
        let cleaned = cleaner.remove_outliers_iqr();
        
        assert_eq!(cleaned.len(), 5);
        assert!(!cleaned.contains(&100.0));
    }

    #[test]
    fn test_summary_stats() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let cleaner = DataCleaner::new(data);
        let stats = cleaner.get_summary_stats();
        
        assert_eq!(stats["mean"], 3.0);
        assert_eq!(stats["median"], 3.0);
        assert_eq!(stats["min"], 1.0);
        assert_eq!(stats["max"], 5.0);
    }
}