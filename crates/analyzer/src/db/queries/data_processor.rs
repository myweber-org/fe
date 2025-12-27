
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub timestamp: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String, timestamp: String) -> Self {
        Self {
            id,
            value,
            category,
            timestamp,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.category.is_empty() && self.value.is_finite()
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

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                continue;
            }

            let id = match parts[0].parse::<u32>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let value = match parts[1].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let category = parts[2].to_string();
            let timestamp = parts[3].to_string();

            let record = DataRecord::new(id, value, category, timestamp);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg = self.calculate_average().unwrap_or(0.0);

        (min, max, avg)
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, 42.5, "A".to_string(), "2023-01-01".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(2, f64::NAN, "B".to_string(), "2023-01-02".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category,timestamp").unwrap();
        writeln!(temp_file, "1,10.5,TypeA,2023-01-01").unwrap();
        writeln!(temp_file, "2,20.3,TypeB,2023-01-02").unwrap();
        writeln!(temp_file, "3,invalid,TypeA,2023-01-03").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        
        assert!(result.is_ok());
        assert_eq!(processor.count_records(), 2);
    }

    #[test]
    fn test_statistics() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "A".to_string(), "2023-01-01".to_string()));
        processor.records.push(DataRecord::new(2, 20.0, "A".to_string(), "2023-01-02".to_string()));
        processor.records.push(DataRecord::new(3, 30.0, "B".to_string(), "2023-01-03".to_string()));

        let stats = processor.get_statistics();
        assert_eq!(stats, (10.0, 30.0, 20.0));
        
        let filtered = processor.filter_by_category("A");
        assert_eq!(filtered.len(), 2);
    }
}
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    InvalidValue,
    EmptyName,
    UnknownCategory,
    DuplicateRecord,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "ID must be greater than zero"),
            DataError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            DataError::EmptyName => write!(f, "Name cannot be empty"),
            DataError::UnknownCategory => write!(f, "Category not recognized"),
            DataError::DuplicateRecord => write!(f, "Record with this ID already exists"),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
    valid_categories: Vec<String>,
}

impl DataProcessor {
    pub fn new(categories: Vec<String>) -> Self {
        DataProcessor {
            records: HashMap::new(),
            valid_categories: categories,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), DataError> {
        if record.id == 0 {
            return Err(DataError::InvalidId);
        }

        if record.name.trim().is_empty() {
            return Err(DataError::EmptyName);
        }

        if record.value < 0.0 || record.value > 1000.0 {
            return Err(DataError::InvalidValue);
        }

        if !self.valid_categories.contains(&record.category) {
            return Err(DataError::UnknownCategory);
        }

        Ok(())
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        self.validate_record(&record)?;

        if self.records.contains_key(&record.id) {
            return Err(DataError::DuplicateRecord);
        }

        self.records.insert(record.id, record);
        Ok(())
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.records.values().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .values()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn transform_values<F>(&mut self, transform_fn: F)
    where
        F: Fn(f64) -> f64,
    {
        for record in self.records.values_mut() {
            record.value = transform_fn(record.value);
        }
    }

    pub fn get_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if self.records.is_empty() {
            return stats;
        }

        let values: Vec<f64> = self.records.values().map(|r| r.value).collect();
        
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg = self.calculate_average();
        
        let variance: f64 = values.iter()
            .map(|&v| (v - avg).powi(2))
            .sum::<f64>() / values.len() as f64;
        
        stats.insert("min".to_string(), min);
        stats.insert("max".to_string(), max);
        stats.insert("average".to_string(), avg);
        stats.insert("variance".to_string(), variance);
        stats.insert("count".to_string(), self.records.len() as f64);

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_processor() -> DataProcessor {
        let categories = vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
        ];
        DataProcessor::new(categories)
    }

    #[test]
    fn test_valid_record() {
        let processor = create_test_processor();
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };
        
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let processor = create_test_processor();
        let record = DataRecord {
            id: 0,
            name: "Test".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };
        
        assert!(matches!(processor.validate_record(&record), Err(DataError::InvalidId)));
    }

    #[test]
    fn test_add_and_retrieve_record() {
        let mut processor = create_test_processor();
        let record = DataRecord {
            id: 1,
            name: "Sample".to_string(),
            value: 50.5,
            category: "B".to_string(),
        };

        assert!(processor.add_record(record.clone()).is_ok());
        assert_eq!(processor.get_record(1).unwrap().name, "Sample");
    }

    #[test]
    fn test_calculate_average() {
        let mut processor = create_test_processor();
        
        let records = vec![
            DataRecord { id: 1, name: "R1".to_string(), value: 10.0, category: "A".to_string() },
            DataRecord { id: 2, name: "R2".to_string(), value: 20.0, category: "A".to_string() },
            DataRecord { id: 3, name: "R3".to_string(), value: 30.0, category: "B".to_string() },
        ];

        for record in records {
            processor.add_record(record).unwrap();
        }

        assert_eq!(processor.calculate_average(), 20.0);
    }

    #[test]
    fn test_transform_values() {
        let mut processor = create_test_processor();
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 10.0,
            category: "A".to_string(),
        };

        processor.add_record(record).unwrap();
        processor.transform_values(|v| v * 2.0);

        assert_eq!(processor.get_record(1).unwrap().value, 20.0);
    }
}