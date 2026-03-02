use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    data: Vec<f64>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { data: Vec::new() }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if let Some(first) = parts.first() {
                if let Ok(value) = first.trim().parse::<f64>() {
                    self.data.push(value);
                }
            }
        }

        Ok(())
    }

    pub fn calculate_mean(&self) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }

        let sum: f64 = self.data.iter().sum();
        Some(sum / self.data.len() as f64)
    }

    pub fn calculate_standard_deviation(&self) -> Option<f64> {
        if self.data.len() < 2 {
            return None;
        }

        let mean = self.calculate_mean()?;
        let variance: f64 = self.data
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / (self.data.len() - 1) as f64;

        Some(variance.sqrt())
    }

    pub fn get_summary(&self) -> SummaryStatistics {
        SummaryStatistics {
            count: self.data.len(),
            mean: self.calculate_mean(),
            std_dev: self.calculate_standard_deviation(),
            min: self.data.iter().copied().reduce(f64::min),
            max: self.data.iter().copied().reduce(f64::max),
        }
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<f64> {
        self.data
            .iter()
            .filter(|&&x| x > threshold)
            .copied()
            .collect()
    }
}

pub struct SummaryStatistics {
    pub count: usize,
    pub mean: Option<f64>,
    pub std_dev: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

impl std::fmt::Display for SummaryStatistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Data Summary:")?;
        writeln!(f, "  Count: {}", self.count)?;
        writeln!(f, "  Mean: {:.4}", self.mean.unwrap_or(f64::NAN))?;
        writeln!(f, "  Std Dev: {:.4}", self.std_dev.unwrap_or(f64::NAN))?;
        writeln!(f, "  Min: {:.4}", self.min.unwrap_or(f64::NAN))?;
        write!(f, "  Max: {:.4}", self.max.unwrap_or(f64::NAN))
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    delimiter: char,
    has_header: bool,
}

impl DataProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        DataProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn process_file<P: AsRef<Path>>(
        &self,
        file_path: P,
        filter_predicate: impl Fn(&[String]) -> bool,
    ) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        let mut records = Vec::new();

        if self.has_header {
            let _ = lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if filter_predicate(&fields) {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn extract_column(&self, data: &[Vec<String>], column_index: usize) -> Vec<String> {
        data.iter()
            .filter_map(|row| row.get(column_index).cloned())
            .collect()
    }

    pub fn calculate_statistics(&self, column_data: &[String]) -> (f64, f64, f64) {
        let numeric_values: Vec<f64> = column_data
            .iter()
            .filter_map(|s| s.parse::<f64>().ok())
            .collect();

        if numeric_values.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = numeric_values.iter().sum();
        let count = numeric_values.len() as f64;
        let mean = sum / count;

        let variance: f64 = numeric_values
            .iter()
            .map(|&value| (value - mean).powi(2))
            .sum::<f64>()
            / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,score").unwrap();
        writeln!(temp_file, "Alice,25,85.5").unwrap();
        writeln!(temp_file, "Bob,30,92.0").unwrap();
        writeln!(temp_file, "Charlie,22,78.5").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor
            .process_file(temp_file.path(), |fields| fields.len() == 3)
            .unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0][0], "Alice");
        assert_eq!(result[1][2], "92.0");

        let ages = processor.extract_column(&result, 1);
        assert_eq!(ages, vec!["25", "30", "22"]);

        let scores = processor.extract_column(&result, 2);
        let stats = processor.calculate_statistics(&scores);
        assert!(stats.0 > 0.0);
    }
}use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataError {
    #[error("Invalid input data")]
    InvalidInput,
    #[error("Transformation failed")]
    TransformationFailed,
    #[error("Validation error: {0}")]
    ValidationError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub value: f64,
    pub timestamp: i64,
}

impl DataRecord {
    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::ValidationError("ID cannot be zero".to_string()));
        }
        if self.value.is_nan() || self.value.is_infinite() {
            return Err(DataError::ValidationError("Value must be finite".to_string()));
        }
        if self.timestamp < 0 {
            return Err(DataError::ValidationError("Timestamp cannot be negative".to_string()));
        }
        Ok(())
    }
}

pub fn process_records(records: Vec<DataRecord>) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records {
        record.validate()?;
        
        let transformed = transform_record(record)?;
        processed.push(transformed);
    }
    
    Ok(processed)
}

fn transform_record(record: DataRecord) -> Result<DataRecord, DataError> {
    if record.value < 0.0 {
        return Err(DataError::TransformationFailed);
    }
    
    let normalized_value = record.value.log10();
    
    Ok(DataRecord {
        id: record.id,
        value: normalized_value,
        timestamp: record.timestamp,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let record = DataRecord {
            id: 1,
            value: 100.0,
            timestamp: 1234567890,
        };
        
        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord {
            id: 0,
            value: f64::NAN,
            timestamp: -1,
        };
        
        assert!(record.validate().is_err());
    }

    #[test]
    fn test_process_records() {
        let records = vec![
            DataRecord {
                id: 1,
                value: 100.0,
                timestamp: 1234567890,
            },
            DataRecord {
                id: 2,
                value: 1000.0,
                timestamp: 1234567891,
            },
        ];
        
        let result = process_records(records);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }
}