use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    value: f64,
    category: String,
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);
        
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        
        Ok(())
    }

    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        let count = self.records.len() as f64;
        let mean = sum / count;

        let variance: f64 = self.records
            .iter()
            .map(|r| (r.value - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,category_a").unwrap();
        writeln!(temp_file, "2,20.3,category_b").unwrap();
        writeln!(temp_file, "3,15.7,category_a").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.get_record_count(), 3);
        
        let stats = processor.calculate_statistics();
        assert!((stats.0 - 15.5).abs() < 0.1);
        
        let filtered = processor.filter_by_category("category_a");
        assert_eq!(filtered.len(), 2);
    }
}
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
        if threshold <= 0.0 || threshold > 100.0 {
            return Err(ValidationError {
                message: format!("Threshold must be between 0 and 100, got {}", threshold),
            });
        }
        Ok(Self { threshold })
    }

    pub fn process_values(&self, values: &[f64]) -> Result<Vec<f64>, ValidationError> {
        if values.is_empty() {
            return Err(ValidationError {
                message: "Input values cannot be empty".to_string(),
            });
        }

        let filtered: Vec<f64> = values
            .iter()
            .filter(|&&v| v >= self.threshold)
            .cloned()
            .collect();

        if filtered.is_empty() {
            return Err(ValidationError {
                message: "No values meet the threshold criteria".to_string(),
            });
        }

        let sum: f64 = filtered.iter().sum();
        let count = filtered.len() as f64;
        let average = sum / count;

        Ok(filtered
            .into_iter()
            .map(|v| v * average)
            .collect())
    }

    pub fn calculate_statistics(&self, values: &[f64]) -> Result<(f64, f64, f64), ValidationError> {
        let processed = self.process_values(values)?;
        
        let min = processed
            .iter()
            .fold(f64::INFINITY, |a, &b| a.min(b));
        let max = processed
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let sum: f64 = processed.iter().sum();
        let mean = sum / processed.len() as f64;

        Ok((min, max, mean))
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
    fn test_invalid_threshold() {
        let processor = DataProcessor::new(0.0);
        assert!(processor.is_err());
        
        let processor = DataProcessor::new(150.0);
        assert!(processor.is_err());
    }

    #[test]
    fn test_process_values() {
        let processor = DataProcessor::new(5.0).unwrap();
        let values = vec![1.0, 6.0, 3.0, 8.0, 10.0];
        let result = processor.process_values(&values);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.len(), 3);
    }

    #[test]
    fn test_empty_input() {
        let processor = DataProcessor::new(5.0).unwrap();
        let result = processor.process_values(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(5.0).unwrap();
        let values = vec![6.0, 8.0, 10.0];
        let result = processor.calculate_statistics(&values);
        assert!(result.is_ok());
        
        let (min, max, mean) = result.unwrap();
        assert!(min > 0.0);
        assert!(max > min);
        assert!(mean >= min && mean <= max);
    }
}
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
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

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn validate_records(&self) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.value.is_finite() && !record.category.is_empty())
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

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,42.5,TypeA").unwrap();
        writeln!(temp_file, "2,37.8,TypeB").unwrap();
        writeln!(temp_file, "3,45.2,TypeA").unwrap();

        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.record_count(), 3);

        let average = processor.calculate_average();
        assert!(average.is_some());
        assert!((average.unwrap() - 41.833).abs() < 0.001);

        let type_a_records = processor.filter_by_category("TypeA");
        assert_eq!(type_a_records.len(), 2);

        let valid_records = processor.validate_records();
        assert_eq!(valid_records.len(), 3);
    }
}