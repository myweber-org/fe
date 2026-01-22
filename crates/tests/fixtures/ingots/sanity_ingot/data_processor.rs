
use csv;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

impl Record {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        Record {
            id,
            name,
            value,
            category,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0 && !self.category.is_empty()
    }

    pub fn process_value(&mut self, multiplier: f64) {
        self.value *= multiplier;
    }
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

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: Record = result?;
            if record.is_valid() {
                self.records.push(record);
            }
        }

        Ok(())
    }

    pub fn add_record(&mut self, record: Record) {
        if record.is_valid() {
            self.records.push(record);
        }
    }

    pub fn process_all(&mut self, multiplier: f64) {
        for record in &mut self.records {
            record.process_value(multiplier);
        }
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn save_to_csv<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::create(path)?;
        let mut wtr = csv::Writer::from_writer(file);

        for record in &self.records {
            wtr.serialize(record)?;
        }

        wtr.flush()?;
        Ok(())
    }

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        let count = self.records.len() as f64;
        if count == 0.0 {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        let avg = sum / count;
        let max = self
            .records
            .iter()
            .map(|r| r.value)
            .fold(f64::NEG_INFINITY, f64::max);

        (sum, avg, max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = Record::new(1, "Test".to_string(), 10.5, "A".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = Record::new(2, "".to_string(), -5.0, "B".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        processor.add_record(Record::new(1, "Item1".to_string(), 10.0, "Cat1".to_string()));
        processor.add_record(Record::new(2, "Item2".to_string(), 20.0, "Cat2".to_string()));

        processor.process_all(2.0);
        let stats = processor.get_statistics();

        assert_eq!(stats.0, 60.0); // Sum: (10*2) + (20*2) = 60
        assert_eq!(stats.1, 30.0); // Average: 60/2 = 30
        assert_eq!(stats.2, 40.0); // Max: 40
    }

    #[test]
    fn test_csv_operations() -> Result<(), Box<dyn Error>> {
        let mut processor = DataProcessor::new();
        processor.add_record(Record::new(1, "Test1".to_string(), 15.5, "CategoryA".to_string()));
        processor.add_record(Record::new(2, "Test2".to_string(), 25.3, "CategoryB".to_string()));

        let temp_file = NamedTempFile::new()?;
        let path = temp_file.path();

        processor.save_to_csv(path)?;

        let mut new_processor = DataProcessor::new();
        new_processor.load_from_csv(path)?;

        assert_eq!(new_processor.records.len(), 2);
        Ok(())
    }
}
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
pub enum DataError {
    InvalidId,
    EmptyValues,
    ValueOutOfRange(f64),
    MissingMetadata(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Invalid record ID"),
            DataError::EmptyValues => write!(f, "Record contains no values"),
            DataError::ValueOutOfRange(val) => write!(f, "Value {} is out of acceptable range", val),
            DataError::MissingMetadata(key) => write!(f, "Missing required metadata: {}", key),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    min_value: f64,
    max_value: f64,
    required_metadata: Vec<String>,
}

impl DataProcessor {
    pub fn new(min_value: f64, max_value: f64, required_metadata: Vec<String>) -> Self {
        DataProcessor {
            min_value,
            max_value,
            required_metadata,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), DataError> {
        if record.id == 0 {
            return Err(DataError::InvalidId);
        }

        if record.values.is_empty() {
            return Err(DataError::EmptyValues);
        }

        for &value in &record.values {
            if value < self.min_value || value > self.max_value {
                return Err(DataError::ValueOutOfRange(value));
            }
        }

        for key in &self.required_metadata {
            if !record.metadata.contains_key(key) {
                return Err(DataError::MissingMetadata(key.clone()));
            }
        }

        Ok(())
    }

    pub fn normalize_values(&self, record: &mut DataRecord) {
        let min_val = record.values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_val = record.values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        
        if max_val > min_val {
            for value in &mut record.values {
                *value = (*value - min_val) / (max_val - min_val);
            }
        }
    }

    pub fn process_records(&self, records: &mut [DataRecord]) -> Vec<Result<(), DataError>> {
        records
            .iter_mut()
            .map(|record| {
                self.validate_record(record)?;
                self.normalize_values(record);
                Ok(())
            })
            .collect()
    }
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();
    
    if records.is_empty() {
        return stats;
    }

    let value_count = records[0].values.len();
    let mut sums = vec![0.0; value_count];
    let mut squares = vec![0.0; value_count];
    let mut mins = vec![f64::INFINITY; value_count];
    let mut maxs = vec![f64::NEG_INFINITY; value_count];

    for record in records {
        for (i, &value) in record.values.iter().enumerate() {
            sums[i] += value;
            squares[i] += value * value;
            mins[i] = mins[i].min(value);
            maxs[i] = maxs[i].max(value);
        }
    }

    let record_count = records.len() as f64;
    
    for i in 0..value_count {
        let mean = sums[i] / record_count;
        let variance = (squares[i] / record_count) - (mean * mean);
        
        stats.insert(format!("mean_{}", i), mean);
        stats.insert(format!("variance_{}", i), variance);
        stats.insert(format!("min_{}", i), mins[i]);
        stats.insert(format!("max_{}", i), maxs[i]);
    }

    stats
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
    fn test_valid_record() {
        let processor = DataProcessor::new(0.0, 100.0, vec!["source".to_string()]);
        let record = create_test_record();
        
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let processor = DataProcessor::new(0.0, 100.0, vec!["source".to_string()]);
        let mut record = create_test_record();
        record.id = 0;
        
        assert!(matches!(processor.validate_record(&record), Err(DataError::InvalidId)));
    }

    #[test]
    fn test_normalization() {
        let processor = DataProcessor::new(0.0, 100.0, vec![]);
        let mut record = DataRecord {
            id: 1,
            values: vec![10.0, 20.0, 30.0],
            metadata: HashMap::new(),
        };
        
        processor.normalize_values(&mut record);
        
        assert_eq!(record.values[0], 0.0);
        assert_eq!(record.values[1], 0.5);
        assert_eq!(record.values[2], 1.0);
    }

    #[test]
    fn test_statistics() {
        let records = vec![
            DataRecord {
                id: 1,
                values: vec![10.0, 20.0],
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 2,
                values: vec![20.0, 40.0],
                metadata: HashMap::new(),
            },
        ];
        
        let stats = calculate_statistics(&records);
        
        assert_eq!(stats.get("mean_0"), Some(&15.0));
        assert_eq!(stats.get("mean_1"), Some(&30.0));
        assert_eq!(stats.get("min_0"), Some(&10.0));
        assert_eq!(stats.get("max_1"), Some(&40.0));
    }
}