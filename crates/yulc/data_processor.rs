
use std::collections::HashMap;
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
    data: HashMap<String, Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: HashMap::new(),
        }
    }

    pub fn add_dataset(&mut self, name: &str, values: Vec<f64>) -> Result<(), ValidationError> {
        if values.is_empty() {
            return Err(ValidationError {
                message: format!("Dataset '{}' cannot be empty", name),
            });
        }

        if values.iter().any(|&x| x.is_nan() || x.is_infinite()) {
            return Err(ValidationError {
                message: format!("Dataset '{}' contains invalid numeric values", name),
            });
        }

        self.data.insert(name.to_string(), values);
        Ok(())
    }

    pub fn calculate_statistics(&self, dataset_name: &str) -> Result<Statistics, ValidationError> {
        let values = self
            .data
            .get(dataset_name)
            .ok_or_else(|| ValidationError {
                message: format!("Dataset '{}' not found", dataset_name),
            })?;

        let count = values.len() as f64;
        let sum: f64 = values.iter().sum();
        let mean = sum / count;

        let variance: f64 = values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>()
            / count;

        let mut sorted_values = values.clone();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let median = if count as usize % 2 == 0 {
            let mid = count as usize / 2;
            (sorted_values[mid - 1] + sorted_values[mid]) / 2.0
        } else {
            sorted_values[count as usize / 2]
        };

        Ok(Statistics {
            count: count as usize,
            mean,
            variance,
            median,
            min: *sorted_values.first().unwrap(),
            max: *sorted_values.last().unwrap(),
        })
    }

    pub fn normalize_dataset(&self, dataset_name: &str) -> Result<Vec<f64>, ValidationError> {
        let values = self
            .data
            .get(dataset_name)
            .ok_or_else(|| ValidationError {
                message: format!("Dataset '{}' not found", dataset_name),
            })?;

        let stats = self.calculate_statistics(dataset_name)?;
        let std_dev = stats.variance.sqrt();

        if std_dev == 0.0 {
            return Ok(vec![0.0; values.len()]);
        }

        let normalized: Vec<f64> = values
            .iter()
            .map(|&x| (x - stats.mean) / std_dev)
            .collect();

        Ok(normalized)
    }

    pub fn list_datasets(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }
}

pub struct Statistics {
    pub count: usize,
    pub mean: f64,
    pub variance: f64,
    pub median: f64,
    pub min: f64,
    pub max: f64,
}

impl fmt::Display for Statistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Statistics:\n  Count: {}\n  Mean: {:.4}\n  Variance: {:.4}\n  Median: {:.4}\n  Min: {:.4}\n  Max: {:.4}",
            self.count, self.mean, self.variance, self.median, self.min, self.max
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_calculate_statistics() {
        let mut processor = DataProcessor::new();
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];

        processor.add_dataset("test_data", values).unwrap();
        let stats = processor.calculate_statistics("test_data").unwrap();

        assert_eq!(stats.count, 5);
        assert_eq!(stats.mean, 3.0);
        assert_eq!(stats.variance, 2.0);
        assert_eq!(stats.median, 3.0);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
    }

    #[test]
    fn test_normalize_dataset() {
        let mut processor = DataProcessor::new();
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];

        processor.add_dataset("test_data", values).unwrap();
        let normalized = processor.normalize_dataset("test_data").unwrap();

        let expected_mean: f64 = normalized.iter().sum::<f64>() / normalized.len() as f64;
        let expected_variance: f64 = normalized
            .iter()
            .map(|&x| (x - expected_mean).powi(2))
            .sum::<f64>()
            / normalized.len() as f64;

        assert!(expected_mean.abs() < 1e-10);
        assert!((expected_variance - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_empty_dataset_error() {
        let mut processor = DataProcessor::new();
        let result = processor.add_dataset("empty", vec![]);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot be empty"));
    }
}