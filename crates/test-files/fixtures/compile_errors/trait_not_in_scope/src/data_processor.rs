use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub value: f64,
    pub timestamp: i64,
}

#[derive(Debug, Error)]
pub enum ProcessingError {
    #[error("Invalid data value: {0}")]
    InvalidValue(String),
    #[error("Timestamp out of range")]
    InvalidTimestamp,
    #[error("Serialization error")]
    SerializationFailed,
}

pub fn validate_record(record: &DataRecord) -> Result<(), ProcessingError> {
    if record.value.is_nan() || record.value.is_infinite() {
        return Err(ProcessingError::InvalidValue(
            "Value must be finite".to_string(),
        ));
    }

    if record.timestamp < 0 {
        return Err(ProcessingError::InvalidTimestamp);
    }

    Ok(())
}

pub fn transform_record(record: &DataRecord, multiplier: f64) -> DataRecord {
    DataRecord {
        id: record.id,
        value: record.value * multiplier,
        timestamp: record.timestamp,
    }
}

pub fn process_records(
    records: &[DataRecord],
    multiplier: f64,
) -> Result<Vec<DataRecord>, ProcessingError> {
    let mut processed = Vec::with_capacity(records.len());

    for record in records {
        validate_record(record)?;
        let transformed = transform_record(record, multiplier);
        processed.push(transformed);
    }

    Ok(processed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_record() {
        let record = DataRecord {
            id: 1,
            value: 42.5,
            timestamp: 1234567890,
        };
        assert!(validate_record(&record).is_ok());
    }

    #[test]
    fn test_validate_invalid_value() {
        let record = DataRecord {
            id: 1,
            value: f64::INFINITY,
            timestamp: 1234567890,
        };
        assert!(validate_record(&record).is_err());
    }

    #[test]
    fn test_transform_record() {
        let record = DataRecord {
            id: 1,
            value: 10.0,
            timestamp: 1000,
        };
        let transformed = transform_record(&record, 2.5);
        assert_eq!(transformed.value, 25.0);
        assert_eq!(transformed.id, record.id);
    }

    #[test]
    fn test_process_records() {
        let records = vec![
            DataRecord {
                id: 1,
                value: 2.0,
                timestamp: 1000,
            },
            DataRecord {
                id: 2,
                value: 3.0,
                timestamp: 2000,
            },
        ];

        let result = process_records(&records, 3.0).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].value, 6.0);
        assert_eq!(result[1].value, 9.0);
    }
}