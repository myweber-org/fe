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
    TransformationError(String),
    ValidationFailed(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationError(msg) => write!(f, "Transformation error: {}", msg),
            ProcessingError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    threshold: f64,
    normalization_factor: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64, normalization_factor: f64) -> Self {
        DataProcessor {
            threshold,
            normalization_factor,
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

        if !record.metadata.contains_key("source") {
            return Err(ProcessingError::ValidationFailed(
                "Missing source metadata".to_string(),
            ));
        }

        Ok(())
    }

    pub fn process_record(&self, mut record: DataRecord) -> Result<DataRecord, ProcessingError> {
        self.validate_record(&record)?;

        let transformed_values: Vec<f64> = record
            .values
            .iter()
            .map(|&v| {
                if v > self.threshold {
                    v / self.normalization_factor
                } else {
                    v * 2.0
                }
            })
            .collect();

        record.values = transformed_values;

        record.metadata.insert(
            "processed_timestamp".to_string(),
            chrono::Utc::now().to_rfc3339(),
        );

        Ok(record)
    }

    pub fn batch_process(
        &self,
        records: Vec<DataRecord>,
    ) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut processed_records = Vec::with_capacity(records.len());
        let mut error_count = 0;

        for record in records {
            match self.process_record(record) {
                Ok(processed) => processed_records.push(processed),
                Err(e) => {
                    error_count += 1;
                    eprintln!("Failed to process record: {}", e);
                }
            }
        }

        if error_count > 0 {
            Err(ProcessingError::TransformationError(format!(
                "Failed to process {} records",
                error_count
            )))
        } else {
            Ok(processed_records)
        }
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        if records.is_empty() {
            return stats;
        }

        let total_values: usize = records.iter().map(|r| r.values.len()).sum();
        let sum: f64 = records
            .iter()
            .flat_map(|r| r.values.iter())
            .copied()
            .sum();

        let mean = sum / total_values as f64;

        let variance: f64 = records
            .iter()
            .flat_map(|r| r.values.iter())
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>()
            / total_values as f64;

        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("total_records".to_string(), records.len() as f64);
        stats.insert("total_values".to_string(), total_values as f64);

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_record() -> DataRecord {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());

        DataRecord {
            id: 1,
            values: vec![10.0, 20.0, 30.0],
            metadata,
        }
    }

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(15.0, 2.0);
        let record = create_test_record();
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let processor = DataProcessor::new(15.0, 2.0);
        let mut record = create_test_record();
        record.metadata.remove("source");
        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_process_record() {
        let processor = DataProcessor::new(15.0, 2.0);
        let record = create_test_record();
        let result = processor.process_record(record);
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert!(processed.metadata.contains_key("processed_timestamp"));
    }

    #[test]
    fn test_batch_processing() {
        let processor = DataProcessor::new(15.0, 2.0);
        let records = vec![create_test_record(), create_test_record()];
        let result = processor.batch_process(records);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }
}