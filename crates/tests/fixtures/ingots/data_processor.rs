
use std::error::Error;
use std::fmt;

#[derive(Debug)]
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
        if threshold <= 0.0 {
            return Err(ValidationError {
                message: "Threshold must be positive".to_string(),
            });
        }

        Ok(DataProcessor { threshold })
    }

    pub fn process_values(&self, values: &[f64]) -> Result<Vec<f64>, ValidationError> {
        if values.is_empty() {
            return Err(ValidationError {
                message: "Input values cannot be empty".to_string(),
            });
        }

        let mut result = Vec::with_capacity(values.len());
        for &value in values {
            if value < 0.0 {
                return Err(ValidationError {
                    message: format!("Negative value {} found in input", value),
                });
            }

            let processed_value = if value > self.threshold {
                value / self.threshold
            } else {
                value * self.threshold
            };

            result.push(processed_value);
        }

        Ok(result)
    }

    pub fn calculate_statistics(&self, values: &[f64]) -> Result<(f64, f64), ValidationError> {
        if values.len() < 2 {
            return Err(ValidationError {
                message: "At least two values required for statistics".to_string(),
            });
        }

        let sum: f64 = values.iter().sum();
        let mean = sum / values.len() as f64;

        let variance: f64 = values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>()
            / (values.len() - 1) as f64;

        Ok((mean, variance.sqrt()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_processor_creation() {
        let processor = DataProcessor::new(10.0);
        assert!(processor.is_ok());
    }

    #[test]
    fn test_invalid_processor_creation() {
        let processor = DataProcessor::new(0.0);
        assert!(processor.is_err());
    }

    #[test]
    fn test_process_values() {
        let processor = DataProcessor::new(5.0).unwrap();
        let values = vec![2.0, 6.0, 10.0];
        let result = processor.process_values(&values);
        assert!(result.is_ok());
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(5.0).unwrap();
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = processor.calculate_statistics(&values);
        assert!(stats.is_ok());
    }
}