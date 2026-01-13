
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ValidationError {
    message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validation error: {}", self.message)
    }
}

impl Error for ValidationError {}

pub struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Result<Self, ValidationError> {
        if threshold < 0.0 || threshold > 1.0 {
            return Err(ValidationError {
                message: format!("Threshold {} must be between 0.0 and 1.0", threshold),
            });
        }
        Ok(Self { threshold })
    }

    pub fn process_values(&self, values: &[f64]) -> Result<Vec<f64>, ValidationError> {
        if values.is_empty() {
            return Err(ValidationError {
                message: "Input values cannot be empty".to_string(),
            });
        }

        let filtered: Vec<f64> = values
            .iter()
            .filter(|&&v| v >= self.threshold)
            .cloned()
            .collect();

        if filtered.is_empty() {
            return Err(ValidationError {
                message: format!(
                    "No values meet the threshold requirement of {}",
                    self.threshold
                ),
            });
        }

        let sum: f64 = filtered.iter().sum();
        let count = filtered.len() as f64;
        let average = sum / count;

        Ok(filtered
            .into_iter()
            .map(|v| (v - average).abs())
            .collect())
    }

    pub fn calculate_statistics(&self, values: &[f64]) -> Result<(f64, f64, f64), ValidationError> {
        let processed = self.process_values(values)?;

        let min = processed
            .iter()
            .fold(f64::INFINITY, |acc, &x| acc.min(x));
        let max = processed
            .iter()
            .fold(f64::NEG_INFINITY, |acc, &x| acc.max(x));
        let sum: f64 = processed.iter().sum();
        let mean = sum / processed.len() as f64;

        Ok((min, max, mean))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_processor_creation() {
        let processor = DataProcessor::new(0.5);
        assert!(processor.is_ok());
    }

    #[test]
    fn test_invalid_processor_creation() {
        let processor = DataProcessor::new(1.5);
        assert!(processor.is_err());
    }

    #[test]
    fn test_process_values() {
        let processor = DataProcessor::new(0.3).unwrap();
        let values = vec![0.1, 0.4, 0.5, 0.2, 0.6];
        let result = processor.process_values(&values);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 3);
    }

    #[test]
    fn test_empty_input() {
        let processor = DataProcessor::new(0.5).unwrap();
        let values: Vec<f64> = vec![];
        let result = processor.process_values(&values);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(0.0).unwrap();
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = processor.calculate_statistics(&values);
        assert!(stats.is_ok());
        let (min, max, mean) = stats.unwrap();
        assert_eq!(min, 2.0);
        assert_eq!(max, 2.0);
        assert_eq!(mean, 2.0);
    }
}