
use std::collections::HashMap;

pub struct DataCleaner {
    data: Vec<f64>,
    threshold: f64,
}

impl DataCleaner {
    pub fn new(data: Vec<f64>, threshold: f64) -> Self {
        DataCleaner { data, threshold }
    }

    pub fn remove_outliers(&self) -> Vec<f64> {
        if self.data.len() < 4 {
            return self.data.clone();
        }

        let mut sorted_data = self.data.clone();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let q1 = Self::calculate_percentile(&sorted_data, 25.0);
        let q3 = Self::calculate_percentile(&sorted_data, 75.0);
        let iqr = q3 - q1;

        let lower_bound = q1 - self.threshold * iqr;
        let upper_bound = q3 + self.threshold * iqr;

        self.data
            .iter()
            .filter(|&&value| value >= lower_bound && value <= upper_bound)
            .cloned()
            .collect()
    }

    fn calculate_percentile(sorted_data: &[f64], percentile: f64) -> f64 {
        let index = (percentile / 100.0) * (sorted_data.len() - 1) as f64;
        let lower_index = index.floor() as usize;
        let upper_index = index.ceil() as usize;

        if lower_index == upper_index {
            sorted_data[lower_index]
        } else {
            let weight = index - lower_index as f64;
            sorted_data[lower_index] * (1.0 - weight) + sorted_data[upper_index] * weight
        }
    }

    pub fn get_summary_stats(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if self.data.is_empty() {
            return stats;
        }

        let sum: f64 = self.data.iter().sum();
        let count = self.data.len() as f64;
        let mean = sum / count;

        let variance: f64 = self.data.iter()
            .map(|&value| (value - mean).powi(2))
            .sum::<f64>() / count;

        let mut sorted = self.data.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("std_dev".to_string(), variance.sqrt());
        stats.insert("min".to_string(), *sorted.first().unwrap_or(&0.0));
        stats.insert("max".to_string(), *sorted.last().unwrap_or(&0.0));
        stats.insert("median".to_string(), Self::calculate_percentile(&sorted, 50.0));

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_outliers() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 100.0];
        let cleaner = DataCleaner::new(data, 1.5);
        let cleaned = cleaner.remove_outliers();
        
        assert_eq!(cleaned.len(), 5);
        assert!(!cleaned.contains(&100.0));
    }

    #[test]
    fn test_summary_stats() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let cleaner = DataCleaner::new(data, 1.5);
        let stats = cleaner.get_summary_stats();
        
        assert_eq!(stats.get("mean").unwrap(), &3.0);
        assert_eq!(stats.get("median").unwrap(), &3.0);
    }
}