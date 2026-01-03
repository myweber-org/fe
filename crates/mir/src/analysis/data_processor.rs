
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

    pub fn process_dataset(&mut self, key: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Empty dataset provided".to_string());
        }

        if let Some(cached) = self.cache.get(key) {
            return Ok(cached.clone());
        }

        let validated = self.validate_data(data)?;
        let normalized = self.normalize_data(&validated);
        let transformed = self.apply_transformations(&normalized);

        self.cache.insert(key.to_string(), transformed.clone());
        Ok(transformed)
    }

    fn validate_data(&self, data: &[f64]) -> Result<Vec<f64>, String> {
        let mut result = Vec::with_capacity(data.len());
        
        for &value in data {
            if value.is_nan() || value.is_infinite() {
                return Err(format!("Invalid numeric value detected: {}", value));
            }
            result.push(value);
        }
        
        Ok(result)
    }

    fn normalize_data(&self, data: &[f64]) -> Vec<f64> {
        if data.len() < 2 {
            return data.to_vec();
        }

        let min = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        if (max - min).abs() < f64::EPSILON {
            return vec![0.5; data.len()];
        }

        data.iter()
            .map(|&x| (x - min) / (max - min))
            .collect()
    }

    fn apply_transformations(&self, data: &[f64]) -> Vec<f64> {
        data.iter()
            .map(|&x| x.ln_1p().exp_m1())
            .collect()
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn cache_stats(&self) -> (usize, usize) {
        let total_items: usize = self.cache.values().map(|v| v.len()).sum();
        (self.cache.len(), total_items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_rejects_invalid_data() {
        let processor = DataProcessor::new();
        let invalid_data = [1.0, f64::NAN, 3.0];
        
        assert!(processor.validate_data(&invalid_data).is_err());
    }

    #[test]
    fn test_normalization_produces_valid_range() {
        let processor = DataProcessor::new();
        let data = [0.0, 50.0, 100.0];
        let normalized = processor.normalize_data(&data);
        
        assert!(normalized.iter().all(|&x| x >= 0.0 && x <= 1.0));
        assert_eq!(normalized[0], 0.0);
        assert_eq!(normalized[2], 1.0);
    }

    #[test]
    fn test_cache_functionality() {
        let mut processor = DataProcessor::new();
        let data = [1.0, 2.0, 3.0];
        
        let first_result = processor.process_dataset("test", &data).unwrap();
        let second_result = processor.process_dataset("test", &data).unwrap();
        
        assert_eq!(first_result, second_result);
        assert_eq!(processor.cache_stats(), (1, 1));
    }
}