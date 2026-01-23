
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
pub enum ProcessingError {
    InvalidData(String),
    TransformationError(String),
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationError(msg) => write!(f, "Transformation error: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    category_stats: HashMap<String, CategoryStats>,
}

#[derive(Debug, Clone)]
pub struct CategoryStats {
    pub category: String,
    pub count: usize,
    pub total_value: f64,
    pub average_value: f64,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            category_stats: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), ProcessingError> {
        self.validate_record(&record)?;
        self.records.push(record.clone());
        self.update_category_stats(&record);
        Ok(())
    }

    pub fn process_records(&mut self) -> Result<Vec<ProcessedRecord>, ProcessingError> {
        let mut processed = Vec::with_capacity(self.records.len());
        
        for record in &self.records {
            let transformed = self.transform_record(record)?;
            processed.push(transformed);
        }
        
        Ok(processed)
    }

    pub fn get_category_stats(&self, category: &str) -> Option<&CategoryStats> {
        self.category_stats.get(category)
    }

    pub fn get_all_stats(&self) -> Vec<CategoryStats> {
        self.category_stats.values().cloned().collect()
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.name.trim().is_empty() {
            return Err(ProcessingError::ValidationError(
                "Record name cannot be empty".to_string()
            ));
        }
        
        if record.value < 0.0 {
            return Err(ProcessingError::ValidationError(
                "Record value cannot be negative".to_string()
            ));
        }
        
        if record.category.trim().is_empty() {
            return Err(ProcessingError::ValidationError(
                "Category cannot be empty".to_string()
            ));
        }
        
        Ok(())
    }

    fn transform_record(&self, record: &DataRecord) -> Result<ProcessedRecord, ProcessingError> {
        let normalized_value = if record.value > 1000.0 {
            record.value / 1000.0
        } else {
            record.value
        };
        
        let processed_name = record.name.to_uppercase();
        
        Ok(ProcessedRecord {
            id: record.id,
            processed_name,
            normalized_value,
            category: record.category.clone(),
            is_premium: normalized_value > 500.0,
        })
    }

    fn update_category_stats(&mut self, record: &DataRecord) {
        let stats = self.category_stats
            .entry(record.category.clone())
            .or_insert_with(|| CategoryStats {
                category: record.category.clone(),
                count: 0,
                total_value: 0.0,
                average_value: 0.0,
            });
        
        stats.count += 1;
        stats.total_value += record.value;
        stats.average_value = stats.total_value / stats.count as f64;
    }
}

#[derive(Debug, Clone)]
pub struct ProcessedRecord {
    pub id: u32,
    pub processed_name: String,
    pub normalized_value: f64,
    pub category: String,
    pub is_premium: bool,
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
    fn test_add_valid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test Record".to_string(),
            value: 100.0,
            category: "Test".to_string(),
        };
        
        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.records.len(), 1);
    }

    #[test]
    fn test_add_invalid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "".to_string(),
            value: 100.0,
            category: "Test".to_string(),
        };
        
        assert!(processor.add_record(record).is_err());
    }

    #[test]
    fn test_category_stats() {
        let mut processor = DataProcessor::new();
        
        let record1 = DataRecord {
            id: 1,
            name: "Record 1".to_string(),
            value: 100.0,
            category: "CategoryA".to_string(),
        };
        
        let record2 = DataRecord {
            id: 2,
            name: "Record 2".to_string(),
            value: 200.0,
            category: "CategoryA".to_string(),
        };
        
        processor.add_record(record1).unwrap();
        processor.add_record(record2).unwrap();
        
        let stats = processor.get_category_stats("CategoryA").unwrap();
        assert_eq!(stats.count, 2);
        assert_eq!(stats.total_value, 300.0);
        assert_eq!(stats.average_value, 150.0);
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
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.count_records(), 0);
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category,timestamp").unwrap();
        writeln!(temp_file, "1,10.5,type_a,2023-01-01").unwrap();
        writeln!(temp_file, "2,20.3,type_b,2023-01-02").unwrap();
        writeln!(temp_file, "3,15.7,type_a,2023-01-03").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.count_records(), 3);
        
        let filtered = processor.filter_by_category("type_a");
        assert_eq!(filtered.len(), 2);
        
        let stats = processor.get_statistics();
        assert_eq!(stats.0, 10.5);
        assert_eq!(stats.1, 20.3);
        
        processor.clear();
        assert_eq!(processor.count_records(), 0);
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
    pub valid: bool,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str) -> Self {
        let valid = value >= 0.0 && value <= 1000.0;
        DataRecord {
            id,
            value,
            category: category.to_string(),
            valid,
        }
    }
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

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                eprintln!("Warning: Invalid format at line {}", line_num + 1);
                continue;
            }
            
            let id = match parts[0].parse::<u32>() {
                Ok(id) => id,
                Err(_) => {
                    eprintln!("Warning: Invalid ID at line {}", line_num + 1);
                    continue;
                }
            };
            
            let value = match parts[1].parse::<f64>() {
                Ok(value) => value,
                Err(_) => {
                    eprintln!("Warning: Invalid value at line {}", line_num + 1);
                    continue;
                }
            };
            
            let category = parts[2].trim().to_string();
            
            self.records.push(DataRecord::new(id, value, &category));
        }
        
        Ok(self.records.len())
    }

    pub fn filter_valid(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| r.valid).collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records: Vec<&DataRecord> = self.filter_valid();
        if valid_records.is_empty() {
            return None;
        }
        
        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&DataRecord>> {
        let mut groups = std::collections::HashMap::new();
        
        for record in &self.records {
            groups
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record);
        }
        
        groups
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn get_records(&self) -> &[DataRecord] {
        &self.records
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 42.5, "test");
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "test");
        assert!(record.valid);
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord::new(2, -10.0, "test");
        assert!(!record.valid);
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.count_records(), 0);
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,100.0,category_a").unwrap();
        writeln!(temp_file, "2,200.0,category_b").unwrap();
        writeln!(temp_file, "3,300.0,category_a").unwrap();
        
        let result = processor.load_from_file(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(processor.count_records(), 3);
        
        let average = processor.calculate_average();
        assert!(average.is_some());
        assert_eq!(average.unwrap(), 200.0);
        
        let groups = processor.group_by_category();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups.get("category_a").unwrap().len(), 2);
        assert_eq!(groups.get("category_b").unwrap().len(), 1);
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

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: u64,
}

impl DataRecord {
    pub fn validate(&self) -> Result<(), ProcessingError> {
        if self.id == 0 {
            return Err(ProcessingError::new("Invalid record ID"));
        }
        if self.value.is_nan() || self.value.is_infinite() {
            return Err(ProcessingError::new("Invalid numeric value"));
        }
        if self.timestamp == 0 {
            return Err(ProcessingError::new("Invalid timestamp"));
        }
        Ok(())
    }
}

pub fn process_records(records: &[DataRecord]) -> Result<Vec<f64>, ProcessingError> {
    let mut results = Vec::with_capacity(records.len());
    
    for record in records {
        record.validate()?;
        
        let processed_value = if record.value < 0.0 {
            record.value.abs()
        } else {
            record.value * 2.0
        };
        
        results.push(processed_value);
    }
    
    Ok(results)
}

pub fn calculate_statistics(values: &[f64]) -> (f64, f64, f64) {
    if values.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    
    let sum: f64 = values.iter().sum();
    let mean = sum / values.len() as f64;
    
    let variance: f64 = values.iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f64>() / values.len() as f64;
    
    let std_dev = variance.sqrt();
    
    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord {
            id: 1,
            value: 42.5,
            timestamp: 1234567890,
        };
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord {
            id: 0,
            value: 42.5,
            timestamp: 1234567890,
        };
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_process_records() {
        let records = vec![
            DataRecord { id: 1, value: 10.0, timestamp: 1000 },
            DataRecord { id: 2, value: -5.0, timestamp: 2000 },
        ];
        
        let result = process_records(&records).unwrap();
        assert_eq!(result, vec![20.0, 5.0]);
    }

    #[test]
    fn test_calculate_statistics() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let (mean, variance, std_dev) = calculate_statistics(&values);
        
        assert_eq!(mean, 3.0);
        assert_eq!(variance, 2.0);
        assert_eq!(std_dev, 2.0_f64.sqrt());
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
pub enum ProcessingError {
    InvalidData(String),
    TransformationError(String),
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationError(msg) => write!(f, "Transformation error: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    normalization_factor: f64,
    validation_threshold: f64,
}

impl DataProcessor {
    pub fn new(normalization_factor: f64, validation_threshold: f64) -> Self {
        DataProcessor {
            normalization_factor,
            validation_threshold,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.values.is_empty() {
            return Err(ProcessingError::ValidationError(
                "Record contains no values".to_string(),
            ));
        }

        for value in &record.values {
            if value.is_nan() || value.is_infinite() {
                return Err(ProcessingError::ValidationError(
                    "Invalid numeric value detected".to_string(),
                ));
            }
        }

        Ok(())
    }

    pub fn normalize_values(&self, record: &mut DataRecord) -> Result<(), ProcessingError> {
        if self.normalization_factor == 0.0 {
            return Err(ProcessingError::TransformationError(
                "Normalization factor cannot be zero".to_string(),
            ));
        }

        for value in &mut record.values {
            *value /= self.normalization_factor;
        }

        Ok(())
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        if records.is_empty() {
            return stats;
        }

        let total_values: usize = records.iter().map(|r| r.values.len()).sum();
        let sum: f64 = records
            .iter()
            .flat_map(|r| r.values.iter())
            .copied()
            .sum();

        stats.insert("total_records".to_string(), records.len() as f64);
        stats.insert("total_values".to_string(), total_values as f64);
        stats.insert("sum".to_string(), sum);

        if total_values > 0 {
            stats.insert("average".to_string(), sum / total_values as f64);
        }

        stats
    }

    pub fn filter_records(
        &self,
        records: Vec<DataRecord>,
    ) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut filtered = Vec::new();

        for record in records {
            match self.validate_record(&record) {
                Ok(_) => {
                    let avg = record.values.iter().sum::<f64>() / record.values.len() as f64;
                    if avg >= self.validation_threshold {
                        filtered.push(record);
                    }
                }
                Err(e) => return Err(e),
            }
        }

        Ok(filtered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new(1.0, 0.5);
        let record = DataRecord {
            id: 1,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_empty_values() {
        let processor = DataProcessor::new(1.0, 0.5);
        let record = DataRecord {
            id: 1,
            values: vec![],
            metadata: HashMap::new(),
        };

        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_normalization() {
        let processor = DataProcessor::new(2.0, 0.5);
        let mut record = DataRecord {
            id: 1,
            values: vec![2.0, 4.0, 6.0],
            metadata: HashMap::new(),
        };

        assert!(processor.normalize_values(&mut record).is_ok());
        assert_eq!(record.values, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_filter_records() {
        let processor = DataProcessor::new(1.0, 2.0);
        let records = vec![
            DataRecord {
                id: 1,
                values: vec![1.0, 2.0, 3.0],
                metadata: HashMap::new(),
            },
            DataRecord {
                id: 2,
                values: vec![3.0, 4.0, 5.0],
                metadata: HashMap::new(),
            },
        ];

        let result = processor.filter_records(records).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, 2);
    }
}