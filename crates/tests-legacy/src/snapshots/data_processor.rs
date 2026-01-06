use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct DataProcessor {
    data: Vec<f64>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { data: Vec::new() }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        for line in reader.lines() {
            let line = line?;
            if let Ok(value) = line.trim().parse::<f64>() {
                self.data.push(value);
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
        let variance: f64 = self.data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / (self.data.len() - 1) as f64;
        
        Some(variance.sqrt())
    }

    pub fn filter_outliers(&self, threshold: f64) -> Vec<f64> {
        if let Some(mean) = self.calculate_mean() {
            if let Some(std_dev) = self.calculate_standard_deviation() {
                return self.data.iter()
                    .filter(|&&x| (x - mean).abs() <= threshold * std_dev)
                    .cloned()
                    .collect();
            }
        }
        self.data.clone()
    }

    pub fn get_summary(&self) -> String {
        format!(
            "Data points: {}, Mean: {:.2}, Std Dev: {:.2}",
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
        writeln!(temp_file, "10.5\n20.3\n15.7\n25.1\n18.9").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        assert_eq!(processor.data.len(), 5);
        assert!(processor.calculate_mean().unwrap() > 0.0);
        assert!(processor.calculate_standard_deviation().unwrap() > 0.0);
        
        let filtered = processor.filter_outliers(2.0);
        assert!(!filtered.is_empty());
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
    category: String,
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidId,
    InvalidName,
    InvalidValue,
    InvalidCategory,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than 0"),
            ValidationError::InvalidName => write!(f, "Name cannot be empty"),
            ValidationError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            ValidationError::InvalidCategory => write!(f, "Category must be one of: A, B, C, D"),
        }
    }
}

impl Error for ValidationError {}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        DataRecord {
            id,
            name,
            value,
            category,
        }
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if self.name.trim().is_empty() {
            return Err(ValidationError::InvalidName);
        }
        
        if self.value < 0.0 || self.value > 1000.0 {
            return Err(ValidationError::InvalidValue);
        }
        
        let valid_categories = ["A", "B", "C", "D"];
        if !valid_categories.contains(&self.category.as_str()) {
            return Err(ValidationError::InvalidCategory);
        }
        
        Ok(())
    }
    
    pub fn transform(&mut self, multiplier: f64) {
        self.value *= multiplier;
        self.name = self.name.to_uppercase();
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    statistics: HashMap<String, f64>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            statistics: HashMap::new(),
        }
    }
    
    pub fn add_record(&mut self, record: DataRecord) -> Result<(), ValidationError> {
        record.validate()?;
        self.records.push(record);
        Ok(())
    }
    
    pub fn process_all(&mut self, multiplier: f64) {
        for record in &mut self.records {
            record.transform(multiplier);
        }
        self.calculate_statistics();
    }
    
    fn calculate_statistics(&mut self) {
        self.statistics.clear();
        
        let count = self.records.len() as f64;
        if count == 0.0 {
            return;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        let avg = sum / count;
        
        let min = self.records.iter()
            .map(|r| r.value)
            .fold(f64::INFINITY, f64::min);
        
        let max = self.records.iter()
            .map(|r| r.value)
            .fold(f64::NEG_INFINITY, f64::max);
        
        self.statistics.insert("count".to_string(), count);
        self.statistics.insert("sum".to_string(), sum);
        self.statistics.insert("average".to_string(), avg);
        self.statistics.insert("minimum".to_string(), min);
        self.statistics.insert("maximum".to_string(), max);
    }
    
    pub fn get_statistics(&self) -> &HashMap<String, f64> {
        &self.statistics
    }
    
    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records.iter()
            .filter(|r| r.category == category)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_record() {
        let record = DataRecord::new(1, "Test".to_string(), 100.0, "A".to_string());
        assert!(record.validate().is_ok());
    }
    
    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(0, "Test".to_string(), 100.0, "A".to_string());
        assert!(matches!(record.validate(), Err(ValidationError::InvalidId)));
    }
    
    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let record1 = DataRecord::new(1, "alpha".to_string(), 50.0, "A".to_string());
        let record2 = DataRecord::new(2, "beta".to_string(), 150.0, "B".to_string());
        
        assert!(processor.add_record(record1).is_ok());
        assert!(processor.add_record(record2).is_ok());
        
        processor.process_all(2.0);
        
        let stats = processor.get_statistics();
        assert_eq!(stats.get("count"), Some(&2.0));
        assert_eq!(stats.get("sum"), Some(&400.0));
    }
}