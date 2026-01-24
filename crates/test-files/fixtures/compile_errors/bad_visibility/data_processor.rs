
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
            DataError::EmptyValues => write!(f, "Record values cannot be empty"),
            DataError::ValueOutOfRange(val) => write!(f, "Value {} is out of valid range", val),
            DataError::MissingMetadata(key) => write!(f, "Missing metadata key: {}", key),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    validation_enabled: bool,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            validation_enabled: true,
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        if self.validation_enabled {
            self.validate_record(&record)?;
        }
        self.records.push(record);
        Ok(())
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), DataError> {
        if record.id == 0 {
            return Err(DataError::InvalidId);
        }

        if record.values.is_empty() {
            return Err(DataError::EmptyValues);
        }

        for &value in &record.values {
            if !value.is_finite() || value < 0.0 || value > 1000.0 {
                return Err(DataError::ValueOutOfRange(value));
            }
        }

        if !record.metadata.contains_key("source") {
            return Err(DataError::MissingMetadata("source".to_string()));
        }

        Ok(())
    }

    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if self.records.is_empty() {
            return stats;
        }

        let total_values: Vec<f64> = self.records
            .iter()
            .flat_map(|r| r.values.clone())
            .collect();

        if !total_values.is_empty() {
            let sum: f64 = total_values.iter().sum();
            let count = total_values.len() as f64;
            let mean = sum / count;
            
            let variance: f64 = total_values
                .iter()
                .map(|&v| (v - mean).powi(2))
                .sum::<f64>() / count;
            
            stats.insert("mean".to_string(), mean);
            stats.insert("variance".to_string(), variance);
            stats.insert("total_records".to_string(), self.records.len() as f64);
            stats.insert("total_values".to_string(), count);
        }

        stats
    }

    pub fn transform_values<F>(&mut self, transform_fn: F) 
    where
        F: Fn(f64) -> f64,
    {
        for record in &mut self.records {
            record.values = record.values
                .iter()
                .map(|&v| transform_fn(v))
                .collect();
        }
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn enable_validation(&mut self, enabled: bool) {
        self.validation_enabled = enabled;
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

impl Default for DataProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record() {
        let mut processor = DataProcessor::new();
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        let record = DataRecord {
            id: 1,
            values: vec![10.5, 20.3, 30.7],
            metadata,
        };

        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.get_record_count(), 1);
    }

    #[test]
    fn test_invalid_id() {
        let processor = DataProcessor::new();
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        let record = DataRecord {
            id: 0,
            values: vec![10.5],
            metadata,
        };

        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let mut processor = DataProcessor::new();
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        
        let records = vec![
            DataRecord {
                id: 1,
                values: vec![10.0, 20.0],
                metadata: metadata.clone(),
            },
            DataRecord {
                id: 2,
                values: vec![30.0, 40.0],
                metadata,
            },
        ];

        for record in records {
            processor.add_record(record).unwrap();
        }

        let stats = processor.calculate_statistics();
        assert_eq!(stats.get("mean").unwrap(), &25.0);
        assert_eq!(stats.get("total_records").unwrap(), &2.0);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<usize, Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut count = 0;
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line_num == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                return Err(format!("Invalid CSV format at line {}", line_num + 1).into());
            }

            let id = parts[0].parse::<u32>()?;
            let value = parts[1].parse::<f64>()?;
            let category = parts[2].to_string();

            if value < 0.0 {
                return Err(format!("Negative value at line {}", line_num + 1).into());
            }

            self.records.push(DataRecord { id, value, category });
            count += 1;
        }

        Ok(count)
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

    pub fn get_max_value(&self) -> Option<&DataRecord> {
        self.records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
    }

    pub fn total_records(&self) -> usize {
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
        writeln!(temp_file, "1,25.5,alpha").unwrap();
        writeln!(temp_file, "2,30.0,beta").unwrap();
        writeln!(temp_file, "3,42.8,alpha").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.total_records(), 3);
        
        let average = processor.calculate_average();
        assert!(average.is_some());
        assert!((average.unwrap() - 32.766).abs() < 0.001);
        
        let alpha_records = processor.filter_by_category("alpha");
        assert_eq!(alpha_records.len(), 2);
        
        let max_record = processor.get_max_value();
        assert!(max_record.is_some());
        assert_eq!(max_record.unwrap().id, 3);
    }
}
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub timestamp: String,
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
            self.validate_record(&record)?;
            self.records.push(record);
        }

        Ok(())
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), String> {
        if record.id == 0 {
            return Err("ID cannot be zero".to_string());
        }
        if record.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if record.value < 0.0 {
            return Err("Value cannot be negative".to_string());
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

    pub fn find_by_id(&self, id: u32) -> Option<&DataRecord> {
        self.records.iter().find(|r| r.id == id)
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.value >= threshold)
            .collect()
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn export_summary(&self) -> String {
        let avg = self.calculate_average().unwrap_or(0.0);
        let max = self.records.iter().map(|r| r.value).fold(0.0, f64::max);
        let min = self.records.iter().map(|r| r.value).fold(f64::INFINITY, f64::min);

        format!(
            "Records: {}, Average: {:.2}, Max: {:.2}, Min: {:.2}",
            self.record_count(),
            avg,
            max,
            min
        )
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
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,timestamp").unwrap();
        writeln!(temp_file, "1,Test1,10.5,2023-01-01").unwrap();
        writeln!(temp_file, "2,Test2,20.3,2023-01-02").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.record_count(), 2);
        
        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 15.4).abs() < 0.001);
        
        let record = processor.find_by_id(1);
        assert!(record.is_some());
        assert_eq!(record.unwrap().name, "Test1");
    }
}