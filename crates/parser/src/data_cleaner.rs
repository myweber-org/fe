use std::collections::HashMap;

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

        let q1_index = sorted_data.len() / 4;
        let q3_index = (3 * sorted_data.len()) / 4;
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
        let count = self.data.len() as f64;
        let mean = sum / count;

        let variance: f64 = self.data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;

        stats.insert("mean".to_string(), mean);
        stats.insert("count".to_string(), count);
        stats.insert("variance".to_string(), variance);
        stats.insert("std_dev".to_string(), variance.sqrt());

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
        
        assert_eq!(stats.get("mean").unwrap(), &3.0);
        assert_eq!(stats.get("count").unwrap(), &5.0);
    }
}use std::collections::HashSet;

pub fn clean_data<T: Eq + std::hash::Hash + Clone>(
    data: Vec<T>,
    invalid_set: &HashSet<T>,
) -> Vec<T> {
    data.into_iter()
        .filter(|item| !invalid_set.contains(item))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_data() {
        let data = vec![1, 2, 3, 4, 5];
        let invalid: HashSet<i32> = [2, 4].iter().cloned().collect();
        let cleaned = clean_data(data, &invalid);
        assert_eq!(cleaned, vec![1, 3, 5]);
    }
}