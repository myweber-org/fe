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

    pub fn process_csv<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines();

        if self.has_header {
            lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line.split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !fields.is_empty() {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn validate_records(&self, records: &[Vec<String>]) -> Vec<usize> {
        let mut invalid_indices = Vec::new();
        
        for (index, record) in records.iter().enumerate() {
            if record.iter().any(|field| field.is_empty()) {
                invalid_indices.push(index);
            }
        }
        
        invalid_indices
    }

    pub fn calculate_column_averages(&self, records: &[Vec<String>]) -> Result<Vec<f64>, Box<dyn Error>> {
        if records.is_empty() {
            return Ok(Vec::new());
        }

        let column_count = records[0].len();
        let mut sums = vec![0.0; column_count];
        let mut counts = vec![0; column_count];

        for record in records {
            for (i, field) in record.iter().enumerate() {
                if let Ok(value) = field.parse::<f64>() {
                    sums[i] += value;
                    counts[i] += 1;
                }
            }
        }

        let averages: Vec<f64> = sums.iter()
            .zip(counts.iter())
            .map(|(&sum, &count)| {
                if count > 0 {
                    sum / count as f64
                } else {
                    0.0
                }
            })
            .collect();

        Ok(averages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000.5").unwrap();
        writeln!(temp_file, "Bob,25,45000.0").unwrap();
        writeln!(temp_file, "Charlie,35,55000.75").unwrap();

        let processor = DataProcessor::new(',', true);
        let records = processor.process_csv(temp_file.path()).unwrap();
        
        assert_eq!(records.len(), 3);
        assert_eq!(records[0], vec!["Alice", "30", "50000.5"]);
    }

    #[test]
    fn test_validation() {
        let records = vec![
            vec!["Alice".to_string(), "30".to_string(), "50000.5".to_string()],
            vec!["Bob".to_string(), "".to_string(), "45000.0".to_string()],
            vec!["Charlie".to_string(), "35".to_string(), "".to_string()],
        ];

        let processor = DataProcessor::new(',', false);
        let invalid_indices = processor.validate_records(&records);
        
        assert_eq!(invalid_indices, vec![1, 2]);
    }

    #[test]
    fn test_average_calculation() {
        let records = vec![
            vec!["10.5".to_string(), "20.0".to_string()],
            vec!["15.5".to_string(), "30.0".to_string()],
            vec!["12.0".to_string(), "25.0".to_string()],
        ];

        let processor = DataProcessor::new(',', false);
        let averages = processor.calculate_column_averages(&records).unwrap();
        
        assert_eq!(averages.len(), 2);
        assert!((averages[0] - 12.666666666666666).abs() < 0.0001);
        assert!((averages[1] - 25.0).abs() < 0.0001);
    }
}
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    name: String,
    value: f64,
    metadata: HashMap<String, String>,
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

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64) -> Self {
        DataRecord {
            id,
            name,
            value,
            metadata: HashMap::new(),
        }
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn validate(&self) -> Result<(), ProcessingError> {
        if self.id == 0 {
            return Err(ProcessingError::ValidationError("ID cannot be zero".to_string()));
        }
        
        if self.name.trim().is_empty() {
            return Err(ProcessingError::ValidationError("Name cannot be empty".to_string()));
        }
        
        if self.value.is_nan() || self.value.is_infinite() {
            return Err(ProcessingError::ValidationError("Value must be a finite number".to_string()));
        }
        
        Ok(())
    }

    pub fn transform(&mut self, multiplier: f64) -> Result<(), ProcessingError> {
        if multiplier <= 0.0 {
            return Err(ProcessingError::TransformationFailed("Multiplier must be positive".to_string()));
        }
        
        self.value *= multiplier;
        Ok(())
    }

    pub fn calculate_score(&self) -> f64 {
        let base_score = self.value * 100.0;
        let metadata_bonus = self.metadata.len() as f64 * 5.0;
        base_score + metadata_bonus
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<f64>, ProcessingError> {
    let mut results = Vec::new();
    
    for record in records.iter_mut() {
        record.validate()?;
        record.transform(1.5)?;
        results.push(record.calculate_score());
    }
    
    Ok(results)
}

pub fn filter_records(records: &[DataRecord], threshold: f64) -> Vec<&DataRecord> {
    records
        .iter()
        .filter(|record| record.value >= threshold)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, "Test".to_string(), 42.0);
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(0, "".to_string(), f64::NAN);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_record_transformation() {
        let mut record = DataRecord::new(1, "Test".to_string(), 10.0);
        assert!(record.transform(2.0).is_ok());
        assert_eq!(record.value, 20.0);
    }

    #[test]
    fn test_calculate_score() {
        let mut record = DataRecord::new(1, "Test".to_string(), 10.0);
        record.add_metadata("category".to_string(), "premium".to_string());
        let score = record.calculate_score();
        assert_eq!(score, 1005.0);
    }
}