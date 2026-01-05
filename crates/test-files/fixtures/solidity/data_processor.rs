
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ValidationError {
    details: String,
}

impl ValidationError {
    fn new(msg: &str) -> ValidationError {
        ValidationError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for ValidationError {
    fn description(&self) -> &str {
        &self.details
    }
}

pub fn validate_numeric_data(data: &[f64]) -> Result<(), ValidationError> {
    if data.is_empty() {
        return Err(ValidationError::new("Data slice cannot be empty"));
    }

    for &value in data {
        if value.is_nan() || value.is_infinite() {
            return Err(ValidationError::new("Data contains invalid numeric values"));
        }
    }

    Ok(())
}

pub fn normalize_data(data: &[f64]) -> Vec<f64> {
    if data.is_empty() {
        return Vec::new();
    }

    let min = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let range = max - min;

    if range == 0.0 {
        return vec![0.0; data.len()];
    }

    data.iter()
        .map(|&x| (x - min) / range)
        .collect()
}

pub fn calculate_statistics(data: &[f64]) -> Result<(f64, f64, f64), ValidationError> {
    validate_numeric_data(data)?;

    let sum: f64 = data.iter().sum();
    let mean = sum / data.len() as f64;

    let variance: f64 = data
        .iter()
        .map(|&value| {
            let diff = mean - value;
            diff * diff
        })
        .sum::<f64>()
        / data.len() as f64;

    let std_dev = variance.sqrt();

    Ok((mean, variance, std_dev))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_numeric_data() {
        let valid_data = vec![1.0, 2.0, 3.0];
        assert!(validate_numeric_data(&valid_data).is_ok());

        let invalid_data = vec![1.0, f64::NAN, 3.0];
        assert!(validate_numeric_data(&invalid_data).is_err());
    }

    #[test]
    fn test_normalize_data() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let normalized = normalize_data(&data);
        
        assert_eq!(normalized.len(), 5);
        assert!((normalized[0] - 0.0).abs() < 1e-10);
        assert!((normalized[4] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_calculate_statistics() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let (mean, variance, std_dev) = calculate_statistics(&data).unwrap();
        
        assert!((mean - 3.0).abs() < 1e-10);
        assert!((variance - 2.0).abs() < 1e-10);
        assert!((std_dev - 1.4142135623730951).abs() < 1e-10);
    }
}
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

        let processed = Self::normalize_data(data)?;
        self.cache.insert(key.to_string(), processed.clone());
        
        Ok(processed)
    }

    fn normalize_data(data: &[f64]) -> Result<Vec<f64>, String> {
        let max_value = data
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        if max_value <= 0.0 {
            return Err("Invalid data range for normalization".to_string());
        }

        let normalized: Vec<f64> = data
            .iter()
            .map(|&x| x / max_value)
            .collect();

        Ok(normalized)
    }

    pub fn calculate_statistics(&self, data: &[f64]) -> (f64, f64, f64) {
        let sum: f64 = data.iter().sum();
        let mean = sum / data.len() as f64;
        
        let variance: f64 = data
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_data() {
        let data = vec![1.0, 2.0, 3.0, 4.0];
        let result = DataProcessor::normalize_data(&data).unwrap();
        assert_eq!(result, vec![0.25, 0.5, 0.75, 1.0]);
    }

    #[test]
    fn test_empty_dataset() {
        let mut processor = DataProcessor::new();
        let result = processor.process_dataset("test", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0, 4.0];
        let (mean, variance, std_dev) = processor.calculate_statistics(&data);
        
        assert!((mean - 2.5).abs() < 1e-10);
        assert!((variance - 1.25).abs() < 1e-10);
        assert!((std_dev - 1.118033988749895).abs() < 1e-10);
    }
}