
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub timestamp: u64,
}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, timestamp: u64) -> Self {
        Self {
            id,
            name,
            value,
            timestamp,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0 && self.timestamp > 0
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

            let id = parts[0].parse::<u32>().unwrap_or(0);
            let name = parts[1].to_string();
            let value = parts[2].parse::<f64>().unwrap_or(0.0);
            let timestamp = parts[3].parse::<u64>().unwrap_or(0);

            let record = DataRecord::new(id, name, value, timestamp);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn filter_by_value(&self, min_value: f64, max_value: f64) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|r| r.value >= min_value && r.value <= max_value)
            .cloned()
            .collect()
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

    pub fn get_records_count(&self) -> usize {
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
        let valid_record = DataRecord::new(1, "test".to_string(), 10.5, 1234567890);
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(2, "".to_string(), -5.0, 0);
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.get_records_count(), 0);

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,timestamp").unwrap();
        writeln!(temp_file, "1,item1,10.5,1000").unwrap();
        writeln!(temp_file, "2,item2,20.3,2000").unwrap();
        writeln!(temp_file, "3,item3,15.7,3000").unwrap();

        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);
        assert_eq!(processor.get_records_count(), 3);

        let filtered = processor.filter_by_value(10.0, 20.0);
        assert_eq!(filtered.len(), 2);

        let stats = processor.calculate_statistics();
        assert!(stats.0 > 0.0);

        processor.clear();
        assert_eq!(processor.get_records_count(), 0);
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
    pub tags: Vec<String>,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    EmptyName,
    NegativeValue,
    DuplicateTag,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than 0"),
            ValidationError::EmptyName => write!(f, "Name cannot be empty"),
            ValidationError::NegativeValue => write!(f, "Value must be non-negative"),
            ValidationError::DuplicateTag => write!(f, "Tags contain duplicates"),
        }
    }
}

impl Error for ValidationError {}

impl DataRecord {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if self.name.trim().is_empty() {
            return Err(ValidationError::EmptyName);
        }
        
        if self.value < 0.0 {
            return Err(ValidationError::NegativeValue);
        }
        
        let mut seen_tags = std::collections::HashSet::new();
        for tag in &self.tags {
            if !seen_tags.insert(tag) {
                return Err(ValidationError::DuplicateTag);
            }
        }
        
        Ok(())
    }
    
    pub fn normalize(&mut self) {
        self.name = self.name.trim().to_string();
        self.tags.sort();
        self.tags.dedup();
    }
}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: HashMap::new(),
        }
    }
    
    pub fn add_record(&mut self, record: DataRecord) -> Result<(), Box<dyn Error>> {
        record.validate()?;
        
        if self.records.contains_key(&record.id) {
            return Err(format!("Record with ID {} already exists", record.id).into());
        }
        
        let mut normalized_record = record.clone();
        normalized_record.normalize();
        
        self.records.insert(normalized_record.id, normalized_record);
        Ok(())
    }
    
    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }
    
    pub fn calculate_total_value(&self) -> f64 {
        self.records.values().map(|r| r.value).sum()
    }
    
    pub fn filter_by_tag(&self, tag: &str) -> Vec<&DataRecord> {
        self.records
            .values()
            .filter(|r| r.tags.iter().any(|t| t == tag))
            .collect()
    }
    
    pub fn remove_record(&mut self, id: u32) -> Option<DataRecord> {
        self.records.remove(&id)
    }
    
    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record() {
        let record = DataRecord {
            id: 1,
            name: "Test Record".to_string(),
            value: 42.5,
            tags: vec!["important".to_string(), "processed".to_string()],
        };
        
        assert!(record.validate().is_ok());
    }
    
    #[test]
    fn test_invalid_id() {
        let record = DataRecord {
            id: 0,
            name: "Test".to_string(),
            value: 10.0,
            tags: vec![],
        };
        
        assert!(matches!(record.validate(), Err(ValidationError::InvalidId)));
    }
    
    #[test]
    fn test_data_processor_operations() {
        let mut processor = DataProcessor::new();
        
        let record1 = DataRecord {
            id: 1,
            name: "First".to_string(),
            value: 100.0,
            tags: vec!["a".to_string(), "b".to_string()],
        };
        
        let record2 = DataRecord {
            id: 2,
            name: "Second".to_string(),
            value: 200.0,
            tags: vec!["b".to_string(), "c".to_string()],
        };
        
        assert!(processor.add_record(record1).is_ok());
        assert!(processor.add_record(record2).is_ok());
        
        assert_eq!(processor.record_count(), 2);
        assert_eq!(processor.calculate_total_value(), 300.0);
        
        let filtered = processor.filter_by_tag("b");
        assert_eq!(filtered.len(), 2);
        
        processor.remove_record(1);
        assert_eq!(processor.record_count(), 1);
    }
}