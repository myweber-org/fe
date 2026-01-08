
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataPoint {
    timestamp: i64,
    value: f64,
    category: String,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidTimestamp,
    InvalidValue,
    EmptyCategory,
    TransformationFailed,
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidTimestamp => write!(f, "Timestamp must be positive"),
            ProcessingError::InvalidValue => write!(f, "Value must be finite"),
            ProcessingError::EmptyCategory => write!(f, "Category cannot be empty"),
            ProcessingError::TransformationFailed => write!(f, "Data transformation failed"),
        }
    }
}

impl Error for ProcessingError {}

impl DataPoint {
    pub fn new(timestamp: i64, value: f64, category: String) -> Result<Self, ProcessingError> {
        if timestamp <= 0 {
            return Err(ProcessingError::InvalidTimestamp);
        }
        
        if !value.is_finite() {
            return Err(ProcessingError::InvalidValue);
        }
        
        if category.trim().is_empty() {
            return Err(ProcessingError::EmptyCategory);
        }
        
        Ok(Self {
            timestamp,
            value,
            category,
        })
    }
    
    pub fn transform(&self, multiplier: f64) -> Result<Self, ProcessingError> {
        if !multiplier.is_finite() || multiplier == 0.0 {
            return Err(ProcessingError::TransformationFailed);
        }
        
        let transformed_value = self.value * multiplier;
        
        DataPoint::new(
            self.timestamp,
            transformed_value,
            self.category.clone()
        )
    }
    
    pub fn normalize(&self, max_value: f64) -> Result<Self, ProcessingError> {
        if max_value <= 0.0 || !max_value.is_finite() {
            return Err(ProcessingError::TransformationFailed);
        }
        
        let normalized_value = self.value / max_value;
        
        DataPoint::new(
            self.timestamp,
            normalized_value,
            self.category.clone()
        )
    }
}

pub fn process_dataset(
    data: Vec<DataPoint>,
    transformation_fn: fn(&DataPoint) -> Result<DataPoint, ProcessingError>
) -> Result<Vec<DataPoint>, ProcessingError> {
    let mut results = Vec::with_capacity(data.len());
    
    for point in data {
        let transformed = transformation_fn(&point)?;
        results.push(transformed);
    }
    
    Ok(results)
}

pub fn calculate_statistics(data: &[DataPoint]) -> (f64, f64, f64) {
    if data.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    
    let sum: f64 = data.iter().map(|p| p.value).sum();
    let count = data.len() as f64;
    let mean = sum / count;
    
    let variance: f64 = data.iter()
        .map(|p| (p.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_datapoint() {
        let point = DataPoint::new(1234567890, 42.5, "temperature".to_string());
        assert!(point.is_ok());
    }
    
    #[test]
    fn test_invalid_timestamp() {
        let point = DataPoint::new(-1, 42.5, "temperature".to_string());
        assert!(matches!(point, Err(ProcessingError::InvalidTimestamp)));
    }
    
    #[test]
    fn test_transform() {
        let point = DataPoint::new(1234567890, 10.0, "pressure".to_string()).unwrap();
        let transformed = point.transform(2.5).unwrap();
        assert_eq!(transformed.value, 25.0);
    }
    
    #[test]
    fn test_normalize() {
        let point = DataPoint::new(1234567890, 75.0, "humidity".to_string()).unwrap();
        let normalized = point.normalize(100.0).unwrap();
        assert_eq!(normalized.value, 0.75);
    }
    
    #[test]
    fn test_statistics() {
        let points = vec![
            DataPoint::new(1, 10.0, "test".to_string()).unwrap(),
            DataPoint::new(2, 20.0, "test".to_string()).unwrap(),
            DataPoint::new(3, 30.0, "test".to_string()).unwrap(),
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&points);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}