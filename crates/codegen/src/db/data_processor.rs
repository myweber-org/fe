
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ProcessingError {
    InvalidInput(String),
    TransformationFailed(String),
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            ProcessingError::TransformationFailed(msg) => write!(f, "Transformation failed: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    threshold: f64,
    max_items: usize,
}

impl DataProcessor {
    pub fn new(threshold: f64, max_items: usize) -> Result<Self, ProcessingError> {
        if threshold < 0.0 || threshold > 1.0 {
            return Err(ProcessingError::InvalidInput(
                "Threshold must be between 0.0 and 1.0".to_string(),
            ));
        }
        if max_items == 0 {
            return Err(ProcessingError::InvalidInput(
                "Max items must be greater than zero".to_string(),
            ));
        }
        Ok(Self {
            threshold,
            max_items,
        })
    }

    pub fn process_data(&self, input: &[f64]) -> Result<Vec<f64>, ProcessingError> {
        if input.len() > self.max_items {
            return Err(ProcessingError::ValidationError(format!(
                "Input length {} exceeds maximum allowed {}",
                input.len(),
                self.max_items
            )));
        }

        let filtered: Vec<f64> = input
            .iter()
            .filter(|&&value| value >= self.threshold)
            .cloned()
            .collect();

        if filtered.is_empty() {
            return Err(ProcessingError::TransformationFailed(
                "No items passed the threshold filter".to_string(),
            ));
        }

        let normalized = self.normalize_values(&filtered)?;
        Ok(normalized)
    }

    fn normalize_values(&self, values: &[f64]) -> Result<Vec<f64>, ProcessingError> {
        let max_value = values
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        if max_value <= 0.0 {
            return Err(ProcessingError::TransformationFailed(
                "Cannot normalize non-positive values".to_string(),
            ));
        }

        let normalized: Vec<f64> = values
            .iter()
            .map(|&value| value / max_value)
            .collect();

        Ok(normalized)
    }

    pub fn calculate_statistics(&self, data: &[f64]) -> Result<(f64, f64, f64), ProcessingError> {
        if data.is_empty() {
            return Err(ProcessingError::InvalidInput(
                "Cannot calculate statistics for empty dataset".to_string(),
            ));
        }

        let sum: f64 = data.iter().sum();
        let mean = sum / data.len() as f64;

        let variance: f64 = data
            .iter()
            .map(|&value| {
                let diff = value - mean;
                diff * diff
            })
            .sum::<f64>()
            / data.len() as f64;

        let std_dev = variance.sqrt();

        Ok((mean, variance, std_dev))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_creation() {
        let processor = DataProcessor::new(0.5, 100).unwrap();
        assert_eq!(processor.threshold, 0.5);
        assert_eq!(processor.max_items, 100);
    }

    #[test]
    fn test_invalid_threshold() {
        let result = DataProcessor::new(1.5, 100);
        assert!(result.is_err());
    }

    #[test]
    fn test_process_data() {
        let processor = DataProcessor::new(0.3, 10).unwrap();
        let input = vec![0.1, 0.4, 0.5, 0.2, 0.8];
        let result = processor.process_data(&input).unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(0.0, 100).unwrap();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let (mean, variance, std_dev) = processor.calculate_statistics(&data).unwrap();
        assert_eq!(mean, 3.0);
        assert_eq!(variance, 2.0);
        assert_eq!(std_dev, 2.0_f64.sqrt());
    }
}