
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct DataSet {
    values: Vec<f64>,
    mean: Option<f64>,
    variance: Option<f64>,
}

impl DataSet {
    pub fn new() -> Self {
        DataSet {
            values: Vec::new(),
            mean: None,
            variance: None,
        }
    }

    pub fn from_csv<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut dataset = DataSet::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(value) = line.trim().parse::<f64>() {
                dataset.values.push(value);
            }
        }

        Ok(dataset)
    }

    pub fn add_value(&mut self, value: f64) {
        self.values.push(value);
        self.mean = None;
        self.variance = None;
    }

    pub fn calculate_mean(&mut self) -> f64 {
        if let Some(mean) = self.mean {
            return mean;
        }

        if self.values.is_empty() {
            self.mean = Some(0.0);
            return 0.0;
        }

        let sum: f64 = self.values.iter().sum();
        let mean = sum / self.values.len() as f64;
        self.mean = Some(mean);
        mean
    }

    pub fn calculate_variance(&mut self) -> f64 {
        if let Some(variance) = self.variance {
            return variance;
        }

        if self.values.len() < 2 {
            self.variance = Some(0.0);
            return 0.0;
        }

        let mean = self.calculate_mean();
        let sum_squared_diff: f64 = self.values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum();
        
        let variance = sum_squared_diff / (self.values.len() - 1) as f64;
        self.variance = Some(variance);
        variance
    }

    pub fn get_values(&self) -> &[f64] {
        &self.values
    }

    pub fn clear(&mut self) {
        self.values.clear();
        self.mean = None;
        self.variance = None;
    }
}

pub fn filter_outliers(data: &[f64], threshold: f64) -> Vec<f64> {
    if data.len() < 3 {
        return data.to_vec();
    }

    let mut temp_dataset = DataSet::new();
    for &value in data {
        temp_dataset.add_value(value);
    }

    let mean = temp_dataset.calculate_mean();
    let std_dev = temp_dataset.calculate_variance().sqrt();

    data.iter()
        .filter(|&&x| (x - mean).abs() <= threshold * std_dev)
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_dataset_operations() {
        let mut dataset = DataSet::new();
        dataset.add_value(10.0);
        dataset.add_value(20.0);
        dataset.add_value(30.0);

        assert_eq!(dataset.calculate_mean(), 20.0);
        assert_eq!(dataset.calculate_variance(), 100.0);
    }

    #[test]
    fn test_csv_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "10.5\n20.3\n30.7\ninvalid\n40.1").unwrap();
        
        let dataset = DataSet::from_csv(temp_file.path()).unwrap();
        assert_eq!(dataset.get_values(), &[10.5, 20.3, 30.7, 40.1]);
    }

    #[test]
    fn test_outlier_filtering() {
        let data = vec![1.0, 2.0, 3.0, 100.0];
        let filtered = filter_outliers(&data, 2.0);
        assert_eq!(filtered, vec![1.0, 2.0, 3.0]);
    }
}use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataProcessor {
    data: Vec<f64>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { data: Vec::new() }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.records() {
            let record = result?;
            for field in record.iter() {
                if let Ok(value) = field.parse::<f64>() {
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

    pub fn filter_outliers(&self, threshold: f64) -> Vec<f64> {
        if let (Some(mean), Some(std_dev)) = (self.calculate_mean(), self.calculate_standard_deviation()) {
            self.data
                .iter()
                .filter(|&&x| (x - mean).abs() <= threshold * std_dev)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_summary(&self) -> String {
        format!(
            "Data points: {}, Mean: {:.4}, Std Dev: {:.4}",
            self.data.len(),
            self.calculate_mean().unwrap_or(0.0),
            self.calculate_standard_deviation().unwrap_or(0.0)
        )
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
        writeln!(temp_file, "1.0,2.0,3.0\n4.0,5.0,6.0").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.data.len(), 6);
        
        assert_eq!(processor.calculate_mean(), Some(3.5));
        assert!(processor.calculate_standard_deviation().unwrap() > 0.0);
        
        let filtered = processor.filter_outliers(2.0);
        assert_eq!(filtered.len(), 6);
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Invalid data format")]
    InvalidFormat,
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Value out of range: {0}")]
    OutOfRange(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::InvalidFormat);
        }
        
        if self.timestamp < 0 {
            return Err(DataError::OutOfRange("timestamp".to_string()));
        }
        
        if self.values.is_empty() {
            return Err(DataError::MissingField("values".to_string()));
        }
        
        Ok(())
    }
    
    pub fn normalize_values(&mut self) {
        if let Some(max) = self.values.iter().copied().reduce(f64::max) {
            if max != 0.0 {
                for value in self.values.iter_mut() {
                    *value /= max;
                }
            }
        }
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<DataRecord>, DataError> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records.iter_mut() {
        record.validate()?;
        record.normalize_values();
        processed.push(record.clone());
    }
    
    Ok(processed)
}

pub fn filter_records(records: &[DataRecord], min_id: u32) -> Vec<DataRecord> {
    records
        .iter()
        .filter(|r| r.id >= min_id)
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };
        
        assert!(valid_record.validate().is_ok());
        
        let invalid_record = DataRecord {
            id: 0,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };
        
        assert!(invalid_record.validate().is_err());
    }
    
    #[test]
    fn test_normalize_values() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![2.0, 4.0, 6.0],
            metadata: HashMap::new(),
        };
        
        record.normalize_values();
        assert_eq!(record.values, vec![1.0/3.0, 2.0/3.0, 1.0]);
    }
}