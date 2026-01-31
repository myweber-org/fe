
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidData(String),
    TransformationFailed(String),
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationFailed(msg) => write!(f, "Transformation failed: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    validation_threshold: f64,
    transformation_factor: f64,
}

impl DataProcessor {
    pub fn new(validation_threshold: f64, transformation_factor: f64) -> Self {
        DataProcessor {
            validation_threshold,
            transformation_factor,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.values.is_empty() {
            return Err(ProcessingError::InvalidData("Empty values vector".to_string()));
        }

        for value in &record.values {
            if value.is_nan() || value.is_infinite() {
                return Err(ProcessingError::InvalidData(
                    "Invalid numeric value detected".to_string(),
                ));
            }
        }

        let sum: f64 = record.values.iter().sum();
        if sum.abs() > self.validation_threshold {
            return Err(ProcessingError::ValidationError(
                format!("Sum {} exceeds threshold {}", sum, self.validation_threshold),
            ));
        }

        Ok(())
    }

    pub fn transform_record(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        self.validate_record(record)?;

        for value in &mut record.values {
            *value *= self.transformation_factor;
            
            if value.is_nan() || value.is_infinite() {
                return Err(ProcessingError::TransformationFailed(
                    "Transformation produced invalid value".to_string(),
                ));
            }
        }

        record.metadata.insert(
            "processed".to_string(),
            "true".to_string(),
        );
        record.metadata.insert(
            "transformation_factor".to_string(),
            self.transformation_factor.to_string(),
        );

        Ok(())
    }

    pub fn batch_process(
        &self,
        records: &mut [DataRecord],
    ) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut processed_records = Vec::with_capacity(records.len());
        let mut errors = Vec::new();

        for record in records.iter_mut() {
            match self.transform_record(record) {
                Ok(_) => processed_records.push(record.clone()),
                Err(e) => errors.push((record.id, e.to_string())),
            }
        }

        if !errors.is_empty() {
            let error_msg = errors
                .iter()
                .map(|(id, msg)| format!("Record {}: {}", id, msg))
                .collect::<Vec<String>>()
                .join("; ");
            return Err(ProcessingError::ProcessingError(error_msg));
        }

        Ok(processed_records)
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if records.is_empty() {
            return stats;
        }

        let all_values: Vec<f64> = records
            .iter()
            .flat_map(|r| r.values.clone())
            .collect();

        let count = all_values.len() as f64;
        let sum: f64 = all_values.iter().sum();
        let mean = sum / count;

        let variance: f64 = all_values
            .iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>()
            / count;

        stats.insert("count".to_string(), count);
        stats.insert("sum".to_string(), sum);
        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("std_dev".to_string(), variance.sqrt());

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(1000.0, 2.0);
        let record = DataRecord {
            id: 1,
            values: vec![10.0, 20.0, 30.0],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_threshold_exceeded() {
        let processor = DataProcessor::new(50.0, 2.0);
        let record = DataRecord {
            id: 1,
            values: vec![100.0, 200.0, 300.0],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transform_record() {
        let processor = DataProcessor::new(1000.0, 2.0);
        let mut record = DataRecord {
            id: 1,
            values: vec![10.0, 20.0, 30.0],
            metadata: HashMap::new(),
        };

        assert!(processor.transform_record(&mut record).is_ok());
        assert_eq!(record.values, vec![20.0, 40.0, 60.0]);
        assert_eq!(record.metadata.get("processed"), Some(&"true".to_string()));
    }
}