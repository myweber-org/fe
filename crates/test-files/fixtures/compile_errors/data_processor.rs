
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
        }
    }

    pub fn process_numeric_data(&mut self, key: &str, values: &[f64]) -> Result<Vec<f64>, String> {
        if values.is_empty() {
            return Err("Empty data array provided".to_string());
        }

        if values.iter().any(|&x| x.is_nan() || x.is_infinite()) {
            return Err("Invalid numeric values detected".to_string());
        }

        let processed: Vec<f64> = values
            .iter()
            .map(|&x| (x * 100.0).round() / 100.0)
            .collect();

        self.cache.insert(key.to_string(), processed.clone());
        Ok(processed)
    }

    pub fn calculate_statistics(&self, key: &str) -> Option<(f64, f64, f64)> {
        self.cache.get(key).map(|data| {
            let sum: f64 = data.iter().sum();
            let count = data.len() as f64;
            let mean = sum / count;

            let variance: f64 = data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / count;
            let std_dev = variance.sqrt();

            (mean, variance, std_dev)
        })
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn get_cached_keys(&self) -> Vec<String> {
        self.cache.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_data_processing() {
        let mut processor = DataProcessor::new();
        let data = vec![1.234, 2.567, 3.891];
        let result = processor.process_numeric_data("test_data", &data);
        
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(processed, vec![1.23, 2.57, 3.89]);
    }

    #[test]
    fn test_invalid_data() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, f64::NAN, 3.0];
        let result = processor.process_numeric_data("invalid", &data);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        processor.process_numeric_data("stats", &data).unwrap();
        
        let stats = processor.calculate_statistics("stats").unwrap();
        assert_eq!(stats.0, 3.0);
    }
}