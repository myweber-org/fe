
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
        }
    }

    pub fn process_numeric_data(&mut self, key: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Empty data array provided".to_string());
        }

        if let Some(cached) = self.cache.get(key) {
            return Ok(cached.clone());
        }

        let processed: Vec<f64> = data
            .iter()
            .filter(|&&x| x.is_finite())
            .map(|&x| x * 2.0)
            .collect();

        if processed.len() < data.len() / 2 {
            return Err("Too many invalid values in input data".to_string());
        }

        self.cache.insert(key.to_string(), processed.clone());
        Ok(processed)
    }

    pub fn calculate_statistics(&self, data: &[f64]) -> Option<(f64, f64, f64)> {
        if data.is_empty() {
            return None;
        }

        let sum: f64 = data.iter().sum();
        let count = data.len() as f64;
        let mean = sum / count;

        let variance: f64 = data
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        Some((mean, variance, std_dev))
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_valid_data() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0];
        let result = processor.process_numeric_data("test", &data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![2.0, 4.0, 6.0]);
    }

    #[test]
    fn test_process_invalid_data() {
        let mut processor = DataProcessor::new();
        let data: Vec<f64> = vec![];
        let result = processor.process_numeric_data("empty", &data);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = processor.calculate_statistics(&data);
        assert!(stats.is_some());
        
        let (mean, variance, std_dev) = stats.unwrap();
        assert_eq!(mean, 3.0);
        assert_eq!(variance, 2.0);
        assert_eq!(std_dev, 2.0_f64.sqrt());
    }
}
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
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if index == 0 {
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
            
            let record = DataRecord {
                id,
                value,
                category: parts[2].to_string(),
                timestamp: parts[3].to_string(),
            };
            
            self.records.push(record);
            count += 1;
        }
        
        Ok(count)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        
        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        
        let values: Vec<f64> = self.records.iter().map(|record| record.value).collect();
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
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.count_records(), 0);
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category,timestamp").unwrap();
        writeln!(temp_file, "1,25.5,type_a,2023-01-01").unwrap();
        writeln!(temp_file, "2,30.0,type_b,2023-01-02").unwrap();
        writeln!(temp_file, "3,15.75,type_a,2023-01-03").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.count_records(), 3);
        
        let filtered = processor.filter_by_category("type_a");
        assert_eq!(filtered.len(), 2);
        
        let stats = processor.get_statistics();
        assert_eq!(stats.0, 15.75);
        assert_eq!(stats.1, 30.0);
        
        processor.clear();
        assert_eq!(processor.count_records(), 0);
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
            DataError::InvalidId => write!(f, "ID must be greater than 0"),
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
    categories: Vec<String>,
}

impl DataProcessor {
    pub fn new(allowed_categories: Vec<String>) -> Self {
        DataProcessor {
            records: HashMap::new(),
            categories: allowed_categories,
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

        if !self.categories.contains(&record.category) {
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
        let count = values.len() as f64;
        let sum: f64 = values.iter().sum();
        let avg = sum / count;

        let variance: f64 = values.iter()
            .map(|v| (v - avg).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();

        stats.insert("count".to_string(), count);
        stats.insert("sum".to_string(), sum);
        stats.insert("average".to_string(), avg);
        stats.insert("variance".to_string(), variance);
        stats.insert("standard_deviation".to_string(), std_dev);

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_addition() {
        let categories = vec!["A".to_string(), "B".to_string()];
        let mut processor = DataProcessor::new(categories);
        
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };

        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.records.len(), 1);
    }

    #[test]
    fn test_invalid_record_rejection() {
        let categories = vec!["A".to_string()];
        let mut processor = DataProcessor::new(categories);
        
        let record = DataRecord {
            id: 0,
            name: "".to_string(),
            value: -10.0,
            category: "B".to_string(),
        };

        assert!(processor.add_record(record).is_err());
    }

    #[test]
    fn test_average_calculation() {
        let categories = vec!["A".to_string()];
        let mut processor = DataProcessor::new(categories);
        
        let records = vec![
            DataRecord { id: 1, name: "R1".to_string(), value: 50.0, category: "A".to_string() },
            DataRecord { id: 2, name: "R2".to_string(), value: 100.0, category: "A".to_string() },
            DataRecord { id: 3, name: "R3".to_string(), value: 150.0, category: "A".to_string() },
        ];

        for record in records {
            processor.add_record(record).unwrap();
        }

        assert_eq!(processor.calculate_average(), 100.0);
    }
}