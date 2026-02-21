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
            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !record.is_empty() {
                records.push(record);
            }
        }

        Ok(records)
    }

    pub fn validate_numeric_column(&self, data: &[Vec<String>], column_index: usize) -> Result<Vec<f64>, String> {
        let mut numeric_values = Vec::new();
        
        for (row_idx, record) in data.iter().enumerate() {
            if column_index >= record.len() {
                return Err(format!("Row {}: Column index out of bounds", row_idx));
            }
            
            match record[column_index].parse::<f64>() {
                Ok(value) => numeric_values.push(value),
                Err(_) => return Err(format!("Row {}: Invalid numeric value '{}'", 
                    row_idx, record[column_index])),
            }
        }
        
        Ok(numeric_values)
    }

    pub fn calculate_statistics(&self, numbers: &[f64]) -> (f64, f64, f64) {
        if numbers.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        
        let sum: f64 = numbers.iter().sum();
        let mean = sum / numbers.len() as f64;
        
        let variance: f64 = numbers.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / numbers.len() as f64;
        
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
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,score").unwrap();
        writeln!(temp_file, "Alice,25,95.5").unwrap();
        writeln!(temp_file, "Bob,30,88.0").unwrap();
        
        let processor = DataProcessor::new(',', true);
        let result = processor.process_csv(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "25", "95.5"]);
    }

    #[test]
    fn test_numeric_validation() {
        let data = vec![
            vec!["10.5".to_string(), "text".to_string()],
            vec!["20.0".to_string(), "more".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let numbers = processor.validate_numeric_column(&data, 0).unwrap();
        
        assert_eq!(numbers, vec![10.5, 20.0]);
    }

    #[test]
    fn test_statistics_calculation() {
        let processor = DataProcessor::new(',', false);
        let numbers = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let (mean, variance, std_dev) = processor.calculate_statistics(&numbers);
        
        assert!((mean - 3.0).abs() < 1e-10);
        assert!((variance - 2.0).abs() < 1e-10);
        assert!((std_dev - 2.0_f64.sqrt()).abs() < 1e-10);
    }
}
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
        DataProcessor { records: Vec::new() }
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

    pub fn calculate_mean(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    pub fn count_records(&self) -> usize {
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
        writeln!(temp_file, "1,10.5,A").unwrap();
        writeln!(temp_file, "2,20.3,B").unwrap();
        writeln!(temp_file, "3,15.7,A").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.count_records(), 3);
        
        let mean = processor.calculate_mean().unwrap();
        assert!((mean - 15.5).abs() < 0.1);
        
        let category_a = processor.filter_by_category("A");
        assert_eq!(category_a.len(), 2);
        
        let max_record = processor.find_max_value().unwrap();
        assert_eq!(max_record.id, 2);
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
pub enum DataError {
    InvalidId,
    InvalidValue,
    MissingField(String),
    ValidationFailed(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Invalid record ID"),
            DataError::InvalidValue => write!(f, "Invalid numeric value"),
            DataError::MissingField(field) => write!(f, "Missing required field: {}", field),
            DataError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl Error for DataError {}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64) -> Result<Self, DataError> {
        if id == 0 {
            return Err(DataError::InvalidId);
        }
        
        if value.is_nan() || value.is_infinite() {
            return Err(DataError::InvalidValue);
        }
        
        if name.trim().is_empty() {
            return Err(DataError::MissingField("name".to_string()));
        }
        
        Ok(Self {
            id,
            name,
            value,
            metadata: HashMap::new(),
        })
    }
    
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
    
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
    
    pub fn transform_value<F>(&mut self, transformer: F) -> Result<(), DataError>
    where
        F: Fn(f64) -> f64,
    {
        let new_value = transformer(self.value);
        
        if new_value.is_nan() || new_value.is_infinite() {
            return Err(DataError::InvalidValue);
        }
        
        self.value = new_value;
        Ok(())
    }
    
    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::InvalidId);
        }
        
        if self.value < 0.0 {
            return Err(DataError::ValidationFailed(
                "Value must be non-negative".to_string()
            ));
        }
        
        if self.name.len() > 100 {
            return Err(DataError::ValidationFailed(
                "Name exceeds maximum length".to_string()
            ));
        }
        
        Ok(())
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }
    
    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        record.validate()?;
        self.records.push(record);
        Ok(())
    }
    
    pub fn process_records<F>(&mut self, processor: F) -> Result<(), DataError>
    where
        F: Fn(&mut DataRecord) -> Result<(), DataError>,
    {
        for record in &mut self.records {
            processor(record)?;
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
    
    pub fn filter_records<F>(&self, predicate: F) -> Vec<&DataRecord>
    where
        F: Fn(&DataRecord) -> bool,
    {
        self.records.iter().filter(|r| predicate(r)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, "Test".to_string(), 42.5);
        assert!(record.is_ok());
    }
    
    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(0, "Test".to_string(), 42.5);
        assert!(matches!(record, Err(DataError::InvalidId)));
    }
    
    #[test]
    fn test_metadata_operations() {
        let mut record = DataRecord::new(1, "Test".to_string(), 42.5).unwrap();
        record.add_metadata("category".to_string(), "sample".to_string());
        
        assert_eq!(record.get_metadata("category"), Some(&"sample".to_string()));
        assert_eq!(record.get_metadata("nonexistent"), None);
    }
    
    #[test]
    fn test_value_transformation() {
        let mut record = DataRecord::new(1, "Test".to_string(), 10.0).unwrap();
        let result = record.transform_value(|x| x * 2.0);
        
        assert!(result.is_ok());
        assert_eq!(record.value, 20.0);
    }
    
    #[test]
    fn test_processor_statistics() {
        let mut processor = DataProcessor::new();
        
        let record1 = DataRecord::new(1, "A".to_string(), 10.0).unwrap();
        let record2 = DataRecord::new(2, "B".to_string(), 20.0).unwrap();
        
        processor.add_record(record1).unwrap();
        processor.add_record(record2).unwrap();
        
        let (mean, variance, std_dev) = processor.calculate_statistics();
        
        assert_eq!(mean, 15.0);
        assert_eq!(variance, 25.0);
        assert_eq!(std_dev, 5.0);
    }
}