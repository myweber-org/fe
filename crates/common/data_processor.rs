
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

    pub fn validate_data(&self, data: &[f64]) -> Result<(), String> {
        if data.is_empty() {
            return Err("Empty data array".to_string());
        }

        for &value in data {
            if !value.is_finite() {
                return Err("Invalid numeric value detected".to_string());
            }
        }

        Ok(())
    }

    pub fn normalize_data(&self, data: &[f64]) -> Vec<f64> {
        if data.is_empty() {
            return Vec::new();
        }

        let min = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        if (max - min).abs() < f64::EPSILON {
            return vec![0.0; data.len()];
        }

        data.iter()
            .map(|&x| (x - min) / (max - min))
            .collect()
    }

    pub fn process_with_cache(&mut self, key: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if let Some(cached) = self.cache.get(key) {
            return Ok(cached.clone());
        }

        self.validate_data(data)?;
        let normalized = self.normalize_data(data);
        self.cache.insert(key.to_string(), normalized.clone());
        
        Ok(normalized)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn cache_stats(&self) -> (usize, usize) {
        let total_keys = self.cache.len();
        let total_values: usize = self.cache.values().map(|v| v.len()).sum();
        (total_keys, total_values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        let processor = DataProcessor::new();
        assert!(processor.validate_data(&[1.0, 2.0, 3.0]).is_ok());
        assert!(processor.validate_data(&[]).is_err());
    }

    #[test]
    fn test_normalization() {
        let processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0];
        let normalized = processor.normalize_data(&data);
        assert_eq!(normalized, vec![0.0, 0.5, 1.0]);
    }

    #[test]
    fn test_cache_operations() {
        let mut processor = DataProcessor::new();
        let data = vec![10.0, 20.0, 30.0];
        
        let result1 = processor.process_with_cache("test", &data);
        assert!(result1.is_ok());
        
        let result2 = processor.process_with_cache("test", &data);
        assert!(result2.is_ok());
        assert_eq!(result1.unwrap(), result2.unwrap());
        
        let stats = processor.cache_stats();
        assert_eq!(stats, (1, 3));
    }
}