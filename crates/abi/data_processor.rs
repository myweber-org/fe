
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

    pub fn process_numeric_data(&mut self, key: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Empty data provided".to_string());
        }

        if let Some(cached) = self.cache.get(key) {
            return Ok(cached.clone());
        }

        let processed = data
            .iter()
            .filter(|&&x| x.is_finite())
            .map(|&x| x * 2.0)
            .collect();

        self.cache.insert(key.to_string(), processed.clone());
        Ok(processed)
    }

    pub fn calculate_statistics(&self, data: &[f64]) -> Option<(f64, f64)> {
        if data.is_empty() {
            return None;
        }

        let sum: f64 = data.iter().sum();
        let mean = sum / data.len() as f64;
        let variance: f64 = data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / data.len() as f64;
        
        Some((mean, variance.sqrt()))
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_numeric_data() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0];
        
        let result = processor.process_numeric_data("test", &data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![2.0, 4.0, 6.0]);
    }

    #[test]
    fn test_empty_data() {
        let mut processor = DataProcessor::new();
        let result = processor.process_numeric_data("empty", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0, 4.0];
        
        let stats = processor.calculate_statistics(&data);
        assert!(stats.is_some());
        
        let (mean, std_dev) = stats.unwrap();
        assert!((mean - 2.5).abs() < 0.0001);
        assert!((std_dev - 1.118033988749895).abs() < 0.0001);
    }
}