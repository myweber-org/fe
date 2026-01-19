
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
            return Err("Empty data provided".to_string());
        }

        if let Some(cached) = self.cache.get(key) {
            return Ok(cached.clone());
        }

        let validated = Self::validate_data(values)?;
        let normalized = Self::normalize_data(&validated);
        let transformed = Self::apply_transformations(&normalized);

        self.cache.insert(key.to_string(), transformed.clone());
        Ok(transformed)
    }

    fn validate_data(values: &[f64]) -> Result<Vec<f64>, String> {
        if values.iter().any(|&x| x.is_nan() || x.is_infinite()) {
            return Err("Invalid numeric values detected".to_string());
        }
        Ok(values.to_vec())
    }

    fn normalize_data(values: &[f64]) -> Vec<f64> {
        let max_value = values.iter().fold(f64::MIN, |a, &b| a.max(b));
        if max_value.abs() < f64::EPSILON {
            return values.to_vec();
        }
        values.iter().map(|&x| x / max_value).collect()
    }

    fn apply_transformations(values: &[f64]) -> Vec<f64> {
        values
            .iter()
            .map(|&x| (x * 100.0).round() / 100.0)
            .collect()
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn get_cache_stats(&self) -> (usize, usize) {
        let total_items: usize = self.cache.values().map(|v| v.len()).sum();
        (self.cache.len(), total_items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0, 4.0];
        
        let result = processor.process_numeric_data("test", &data);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.len(), 4);
        assert!(processed[0] >= 0.0 && processed[0] <= 1.0);
    }

    #[test]
    fn test_invalid_data() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, f64::NAN, 3.0];
        
        let result = processor.process_numeric_data("invalid", &data);
        assert!(result.is_err());
    }
}