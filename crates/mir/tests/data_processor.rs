
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataRecord {
    id: u32,
    value: f64,
    timestamp: String,
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.value >= threshold)
            .collect()
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.record_count(), 0);
        assert_eq!(processor.calculate_average(), None);
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,timestamp").unwrap();
        writeln!(temp_file, "1,42.5,2023-01-01T10:00:00Z").unwrap();
        writeln!(temp_file, "2,37.8,2023-01-01T11:00:00Z").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        
        assert!(result.is_ok());
        assert_eq!(processor.record_count(), 2);
        
        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 40.15).abs() < 0.01);
    }

    #[test]
    fn test_threshold_filter() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord {
            id: 1,
            value: 30.0,
            timestamp: "2023-01-01T10:00:00Z".to_string(),
        });
        processor.records.push(DataRecord {
            id: 2,
            value: 50.0,
            timestamp: "2023-01-01T11:00:00Z".to_string(),
        });

        let filtered = processor.filter_by_threshold(40.0);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 2);
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ProcessingError {
    message: String,
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Processing error: {}", self.message)
    }
}

impl Error for ProcessingError {}

impl ProcessingError {
    pub fn new(msg: &str) -> Self {
        ProcessingError {
            message: msg.to_string(),
        }
    }
}

pub struct DataProcessor {
    threshold: f64,
    multiplier: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64, multiplier: f64) -> Result<Self, ProcessingError> {
        if threshold <= 0.0 {
            return Err(ProcessingError::new("Threshold must be positive"));
        }
        if multiplier <= 0.0 {
            return Err(ProcessingError::new("Multiplier must be positive"));
        }
        
        Ok(DataProcessor {
            threshold,
            multiplier,
        })
    }
    
    pub fn process_value(&self, value: f64) -> Result<f64, ProcessingError> {
        if value < 0.0 {
            return Err(ProcessingError::new("Value cannot be negative"));
        }
        
        if value > self.threshold {
            let adjusted = value * self.multiplier;
            Ok(adjusted.ln())
        } else {
            Ok(value.sqrt())
        }
    }
    
    pub fn batch_process(&self, values: &[f64]) -> Vec<Result<f64, ProcessingError>> {
        values.iter()
            .map(|&v| self.process_value(v))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_processor_creation() {
        let processor = DataProcessor::new(10.0, 2.0);
        assert!(processor.is_ok());
    }
    
    #[test]
    fn test_invalid_threshold() {
        let processor = DataProcessor::new(0.0, 2.0);
        assert!(processor.is_err());
    }
    
    #[test]
    fn test_process_value_below_threshold() {
        let processor = DataProcessor::new(10.0, 2.0).unwrap();
        let result = processor.process_value(9.0);
        assert!(result.is_ok());
        assert!((result.unwrap() - 3.0).abs() < 0.0001);
    }
    
    #[test]
    fn test_process_value_above_threshold() {
        let processor = DataProcessor::new(10.0, 2.0).unwrap();
        let result = processor.process_value(20.0);
        assert!(result.is_ok());
        let expected = (20.0 * 2.0).ln();
        assert!((result.unwrap() - expected).abs() < 0.0001);
    }
    
    #[test]
    fn test_negative_value() {
        let processor = DataProcessor::new(10.0, 2.0).unwrap();
        let result = processor.process_value(-5.0);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_batch_processing() {
        let processor = DataProcessor::new(10.0, 2.0).unwrap();
        let values = vec![4.0, 16.0, -2.0, 25.0];
        let results = processor.batch_process(&values);
        
        assert_eq!(results.len(), 4);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
        assert!(results[2].is_err());
        assert!(results[3].is_ok());
    }
}