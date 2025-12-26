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

    pub fn process_data(&self, input: &[f64]) -> Result<Vec<f64>, ValidationError> {
        if input.is_empty() {
            return Err(ValidationError {
                message: "Input data cannot be empty".to_string(),
            });
        }

        let normalized: Vec<f64> = input
            .iter()
            .map(|&value| {
                if value.is_nan() || value.is_infinite() {
                    0.0
                } else {
                    value
                }
            })
            .collect();

        let max_value = normalized
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        if max_value.abs() < f64::EPSILON {
            return Err(ValidationError {
                message: "All values are zero or invalid".to_string(),
            });
        }

        let processed: Vec<f64> = normalized
            .iter()
            .map(|&value| {
                let scaled = value / max_value;
                if scaled >= self.threshold {
                    scaled.log10()
                } else {
                    scaled * scaled
                }
            })
            .collect();

        Ok(processed)
    }

    pub fn calculate_statistics(&self, data: &[f64]) -> Result<(f64, f64, f64), ValidationError> {
        if data.len() < 2 {
            return Err(ValidationError {
                message: "Insufficient data for statistics calculation".to_string(),
            });
        }

        let sum: f64 = data.iter().sum();
        let mean = sum / data.len() as f64;

        let variance: f64 = data
            .iter()
            .map(|value| {
                let diff = value - mean;
                diff * diff
            })
            .sum::<f64>()
            / (data.len() - 1) as f64;

        let std_dev = variance.sqrt();

        Ok((mean, variance, std_dev))
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
    fn test_process_data() {
        let processor = DataProcessor::new(0.7).unwrap();
        let input = vec![1.0, 2.0, 3.0, 4.0];
        let result = processor.process_data(&input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 4);
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(0.5).unwrap();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = processor.calculate_statistics(&data);
        assert!(stats.is_ok());
        let (mean, variance, std_dev) = stats.unwrap();
        assert!((mean - 3.0).abs() < 0.0001);
        assert!((variance - 2.5).abs() < 0.0001);
        assert!((std_dev - 1.5811).abs() < 0.0001);
    }
}