
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u32, values: Vec<f64>) -> Self {
        Self {
            id,
            values,
            metadata: HashMap::new(),
        }
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        if self.id == 0 {
            return Err("Invalid record ID".into());
        }
        
        if self.values.is_empty() {
            return Err("Empty values vector".into());
        }

        for value in &self.values {
            if value.is_nan() || value.is_infinite() {
                return Err("Invalid numeric value detected".into());
            }
        }

        Ok(())
    }

    pub fn normalize(&mut self) {
        if let Some(max) = self.values.iter().copied().reduce(f64::max) {
            if max != 0.0 {
                for value in &mut self.values {
                    *value /= max;
                }
            }
        }
    }

    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        let count = self.values.len() as f64;
        let sum: f64 = self.values.iter().sum();
        let mean = sum / count;

        let variance: f64 = self.values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let mut processed = Vec::new();

    for record in records {
        record.validate()?;
        let mut processed_record = record.clone();
        processed_record.normalize();
        processed.push(processed_record);
    }

    Ok(processed)
}

pub fn aggregate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();
    let mut total_mean = 0.0;
    let mut total_variance = 0.0;
    let mut count = 0;

    for record in records {
        let (mean, variance, std_dev) = record.calculate_statistics();
        total_mean += mean;
        total_variance += variance;
        count += 1;

        stats.insert(format!("record_{}_mean", record.id), mean);
        stats.insert(format!("record_{}_std_dev", record.id), std_dev);
    }

    if count > 0 {
        stats.insert("overall_mean".to_string(), total_mean / count as f64);
        stats.insert("overall_variance".to_string(), total_variance / count as f64);
    }

    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, vec![1.0, 2.0, 3.0]);
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(0, vec![1.0, 2.0]);
        assert!(invalid_record.validate().is_err());

        let nan_record = DataRecord::new(2, vec![1.0, f64::NAN]);
        assert!(nan_record.validate().is_err());
    }

    #[test]
    fn test_normalization() {
        let mut record = DataRecord::new(1, vec![1.0, 2.0, 3.0]);
        record.normalize();
        assert_eq!(record.values, vec![1.0/3.0, 2.0/3.0, 1.0]);
    }

    #[test]
    fn test_statistics_calculation() {
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0]);
        let (mean, variance, std_dev) = record.calculate_statistics();
        
        assert!((mean - 2.0).abs() < 1e-10);
        assert!((variance - 2.0/3.0).abs() < 1e-10);
        assert!((std_dev - (2.0/3.0 as f64).sqrt()).abs() < 1e-10);
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
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
    config: ProcessingConfig,
}

#[derive(Debug, Clone)]
pub struct ProcessingConfig {
    pub max_values: usize,
    pub min_timestamp: i64,
    pub max_timestamp: i64,
    pub allowed_metadata_keys: Vec<String>,
}

impl DataProcessor {
    pub fn new(config: ProcessingConfig) -> Self {
        DataProcessor { config }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.values.len() > self.config.max_values {
            return Err(ProcessingError::ValidationError(format!(
                "Too many values: {} > {}",
                record.values.len(),
                self.config.max_values
            )));
        }

        if record.timestamp < self.config.min_timestamp
            || record.timestamp > self.config.max_timestamp
        {
            return Err(ProcessingError::ValidationError(format!(
                "Timestamp {} out of range [{}, {}]",
                record.timestamp, self.config.min_timestamp, self.config.max_timestamp
            )));
        }

        for key in record.metadata.keys() {
            if !self.config.allowed_metadata_keys.contains(key) {
                return Err(ProcessingError::ValidationError(format!(
                    "Metadata key '{}' not allowed",
                    key
                )));
            }
        }

        Ok(())
    }

    pub fn transform_record(
        &self,
        record: &DataRecord,
    ) -> Result<TransformedRecord, ProcessingError> {
        self.validate_record(record)?;

        let sum: f64 = record.values.iter().sum();
        let avg = if !record.values.is_empty() {
            sum / record.values.len() as f64
        } else {
            0.0
        };

        let variance: f64 = if record.values.len() > 1 {
            let mean = avg;
            record
                .values
                .iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>()
                / (record.values.len() - 1) as f64
        } else {
            0.0
        };

        let metadata_count = record.metadata.len();

        Ok(TransformedRecord {
            original_id: record.id,
            timestamp: record.timestamp,
            value_count: record.values.len(),
            value_sum: sum,
            value_average: avg,
            value_variance: variance,
            metadata_count,
            processed_at: chrono::Utc::now().timestamp(),
        })
    }

    pub fn process_batch(
        &self,
        records: Vec<DataRecord>,
    ) -> Result<Vec<TransformedRecord>, ProcessingError> {
        let mut results = Vec::with_capacity(records.len());
        let mut errors = Vec::new();

        for (index, record) in records.into_iter().enumerate() {
            match self.transform_record(&record) {
                Ok(transformed) => results.push(transformed),
                Err(e) => errors.push((index, e)),
            }
        }

        if !errors.is_empty() {
            let error_msg = errors
                .iter()
                .map(|(idx, err)| format!("Record {}: {}", idx, err))
                .collect::<Vec<String>>()
                .join("; ");
            return Err(ProcessingError::ProcessingError(format!(
                "Batch processing failed with errors: {}",
                error_msg
            )));
        }

        Ok(results)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformedRecord {
    pub original_id: u64,
    pub timestamp: i64,
    pub value_count: usize,
    pub value_sum: f64,
    pub value_average: f64,
    pub value_variance: f64,
    pub metadata_count: usize,
    pub processed_at: i64,
}

impl ProcessingError {
    fn ProcessingError(msg: String) -> Self {
        ProcessingError::TransformationError(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> ProcessingConfig {
        ProcessingConfig {
            max_values: 10,
            min_timestamp: 0,
            max_timestamp: 1000000000,
            allowed_metadata_keys: vec!["source".to_string(), "type".to_string()],
        }
    }

    fn create_valid_record() -> DataRecord {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());

        DataRecord {
            id: 1,
            timestamp: 123456789,
            values: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            metadata,
        }
    }

    #[test]
    fn test_valid_record_validation() {
        let processor = DataProcessor::new(create_test_config());
        let record = create_valid_record();
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_invalid_timestamp() {
        let processor = DataProcessor::new(create_test_config());
        let mut record = create_valid_record();
        record.timestamp = -1;
        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_too_many_values() {
        let processor = DataProcessor::new(create_test_config());
        let mut record = create_valid_record();
        record.values = vec![1.0; 11];
        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_invalid_metadata_key() {
        let processor = DataProcessor::new(create_test_config());
        let mut record = create_valid_record();
        record.metadata.insert("invalid".to_string(), "value".to_string());
        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_record_transformation() {
        let processor = DataProcessor::new(create_test_config());
        let record = create_valid_record();
        let transformed = processor.transform_record(&record).unwrap();

        assert_eq!(transformed.original_id, 1);
        assert_eq!(transformed.timestamp, 123456789);
        assert_eq!(transformed.value_count, 5);
        assert_eq!(transformed.value_sum, 15.0);
        assert_eq!(transformed.value_average, 3.0);
        assert_eq!(transformed.metadata_count, 1);
    }

    #[test]
    fn test_batch_processing() {
        let processor = DataProcessor::new(create_test_config());
        let records = vec![create_valid_record(), create_valid_record()];
        let results = processor.process_batch(records).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].original_id, 1);
        assert_eq!(results[1].original_id, 1);
    }

    #[test]
    fn test_batch_processing_with_error() {
        let processor = DataProcessor::new(create_test_config());
        let mut invalid_record = create_valid_record();
        invalid_record.timestamp = -1;

        let records = vec![create_valid_record(), invalid_record];
        let result = processor.process_batch(records);

        assert!(result.is_err());
    }
}
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataProcessor {
    data: Vec<Vec<String>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { data: Vec::new() }
    }

    pub fn load_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        
        for result in rdr.records() {
            let record = result?;
            let row: Vec<String> = record.iter().map(|field| field.to_string()).collect();
            self.data.push(row);
        }
        
        Ok(())
    }

    pub fn validate_data(&self) -> bool {
        if self.data.is_empty() {
            return false;
        }
        
        let header_len = self.data[0].len();
        for row in &self.data[1..] {
            if row.len() != header_len {
                return false;
            }
        }
        
        true
    }

    pub fn get_row_count(&self) -> usize {
        self.data.len()
    }

    pub fn get_column_count(&self) -> usize {
        if self.data.is_empty() {
            0
        } else {
            self.data[0].len()
        }
    }

    pub fn filter_rows<F>(&self, predicate: F) -> Vec<Vec<String>>
    where
        F: Fn(&[String]) -> bool,
    {
        self.data.iter()
            .filter(|row| predicate(row))
            .cloned()
            .collect()
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
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        
        let result = processor.load_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.get_row_count(), 3);
        assert_eq!(processor.get_column_count(), 3);
        assert!(processor.validate_data());
        
        let filtered = processor.filter_rows(|row| {
            row.get(1).and_then(|age| age.parse::<i32>().ok()).map_or(false, |age| age > 25)
        });
        
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0][0], "Alice");
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        Self {
            id,
            name,
            value,
            category,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0 && !self.category.is_empty()
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

            let name = parts[1].to_string();
            let value = match parts[2].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let category = parts[3].to_string();

            let record = DataRecord::new(id, name, value, category);
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

    pub fn find_max_value(&self) -> Option<&DataRecord> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn get_records(&self) -> &[DataRecord] {
        &self.records
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
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, "Test".to_string(), 10.5, "A".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(2, "".to_string(), -5.0, "".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,Item1,10.5,CategoryA").unwrap();
        writeln!(temp_file, "2,Item2,20.3,CategoryB").unwrap();
        writeln!(temp_file, "3,Item3,15.7,CategoryA").unwrap();

        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);

        let category_a = processor.filter_by_category("CategoryA");
        assert_eq!(category_a.len(), 2);

        let avg = processor.calculate_average().unwrap();
        assert!((avg - 15.5).abs() < 0.1);

        let max_record = processor.find_max_value().unwrap();
        assert_eq!(max_record.id, 2);
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
    MissingField,
    DuplicateRecord,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Invalid record ID"),
            DataError::InvalidValue => write!(f, "Invalid numeric value"),
            DataError::MissingField => write!(f, "Required field is missing"),
            DataError::DuplicateRecord => write!(f, "Duplicate record detected"),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
    category_totals: HashMap<String, f64>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: HashMap::new(),
            category_totals: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        if record.id == 0 {
            return Err(DataError::InvalidId);
        }

        if record.value < 0.0 || record.value.is_nan() {
            return Err(DataError::InvalidValue);
        }

        if record.name.is_empty() || record.category.is_empty() {
            return Err(DataError::MissingField);
        }

        if self.records.contains_key(&record.id) {
            return Err(DataError::DuplicateRecord);
        }

        self.records.insert(record.id, record.clone());
        
        let total = self.category_totals
            .entry(record.category.clone())
            .or_insert(0.0);
        *total += record.value;

        Ok(())
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn get_category_total(&self, category: &str) -> f64 {
        *self.category_totals.get(category).unwrap_or(&0.0)
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
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn transform_values<F>(&mut self, transform_fn: F) 
    where
        F: Fn(f64) -> f64,
    {
        for record in self.records.values_mut() {
            record.value = transform_fn(record.value);
        }
        
        self.recalculate_totals();
    }

    fn recalculate_totals(&mut self) {
        self.category_totals.clear();
        
        for record in self.records.values() {
            let total = self.category_totals
                .entry(record.category.clone())
                .or_insert(0.0);
            *total += record.value;
        }
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn validate_all(&self) -> Vec<DataError> {
        let mut errors = Vec::new();
        
        for record in self.records.values() {
            if record.id == 0 {
                errors.push(DataError::InvalidId);
            }
            
            if record.value < 0.0 || record.value.is_nan() {
                errors.push(DataError::InvalidValue);
            }
            
            if record.name.is_empty() || record.category.is_empty() {
                errors.push(DataError::MissingField);
            }
        }
        
        errors
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
            name: "Test".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };
        
        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.record_count(), 1);
    }

    #[test]
    fn test_duplicate_record() {
        let mut processor = DataProcessor::new();
        let record1 = DataRecord {
            id: 1,
            name: "Test1".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };
        
        let record2 = DataRecord {
            id: 1,
            name: "Test2".to_string(),
            value: 200.0,
            category: "B".to_string(),
        };
        
        assert!(processor.add_record(record1).is_ok());
        assert!(processor.add_record(record2).is_err());
    }

    #[test]
    fn test_category_total() {
        let mut processor = DataProcessor::new();
        
        let records = vec![
            DataRecord {
                id: 1,
                name: "Item1".to_string(),
                value: 50.0,
                category: "Electronics".to_string(),
            },
            DataRecord {
                id: 2,
                name: "Item2".to_string(),
                value: 75.0,
                category: "Electronics".to_string(),
            },
            DataRecord {
                id: 3,
                name: "Item3".to_string(),
                value: 25.0,
                category: "Books".to_string(),
            },
        ];
        
        for record in records {
            processor.add_record(record).unwrap();
        }
        
        assert_eq!(processor.get_category_total("Electronics"), 125.0);
        assert_eq!(processor.get_category_total("Books"), 25.0);
    }

    #[test]
    fn test_value_transformation() {
        let mut processor = DataProcessor::new();
        
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };
        
        processor.add_record(record).unwrap();
        
        processor.transform_values(|x| x * 1.1);
        
        let updated_record = processor.get_record(1).unwrap();
        assert_eq!(updated_record.value, 110.0);
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    data: HashMap<String, Vec<f64>>,
    validation_rules: ValidationRules,
}

pub struct ValidationRules {
    min_value: f64,
    max_value: f64,
    required_keys: Vec<String>,
}

impl DataProcessor {
    pub fn new(rules: ValidationRules) -> Self {
        DataProcessor {
            data: HashMap::new(),
            validation_rules: rules,
        }
    }

    pub fn add_dataset(&mut self, key: String, values: Vec<f64>) -> Result<(), String> {
        if !self.validation_rules.required_keys.contains(&key) {
            return Err(format!("Key '{}' is not in required keys list", key));
        }

        for &value in &values {
            if value < self.validation_rules.min_value || value > self.validation_rules.max_value {
                return Err(format!("Value {} is outside allowed range [{}, {}]", 
                    value, self.validation_rules.min_value, self.validation_rules.max_value));
            }
        }

        self.data.insert(key, values);
        Ok(())
    }

    pub fn calculate_statistics(&self) -> HashMap<String, Stats> {
        let mut results = HashMap::new();
        
        for (key, values) in &self.data {
            if values.is_empty() {
                continue;
            }

            let sum: f64 = values.iter().sum();
            let count = values.len() as f64;
            let mean = sum / count;
            
            let variance: f64 = values.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count;
            
            let std_dev = variance.sqrt();
            
            let mut sorted_values = values.clone();
            sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
            
            let median = if count as usize % 2 == 0 {
                let mid = count as usize / 2;
                (sorted_values[mid - 1] + sorted_values[mid]) / 2.0
            } else {
                sorted_values[count as usize / 2]
            };

            results.insert(key.clone(), Stats {
                mean,
                median,
                std_dev,
                min: *sorted_values.first().unwrap(),
                max: *sorted_values.last().unwrap(),
                count: values.len(),
            });
        }
        
        results
    }

    pub fn normalize_data(&mut self) {
        for values in self.data.values_mut() {
            if values.is_empty() {
                continue;
            }

            let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            
            if (max - min).abs() > f64::EPSILON {
                for value in values.iter_mut() {
                    *value = (*value - min) / (max - min);
                }
            }
        }
    }

    pub fn get_data_summary(&self) -> DataSummary {
        let mut total_values = 0;
        let mut all_values = Vec::new();
        
        for values in self.data.values() {
            total_values += values.len();
            all_values.extend(values);
        }

        DataSummary {
            dataset_count: self.data.len(),
            total_values,
            keys: self.data.keys().cloned().collect(),
        }
    }
}

pub struct Stats {
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub count: usize,
}

pub struct DataSummary {
    pub dataset_count: usize,
    pub total_values: usize,
    pub keys: Vec<String>,
}

impl ValidationRules {
    pub fn new(min_value: f64, max_value: f64, required_keys: Vec<String>) -> Self {
        ValidationRules {
            min_value,
            max_value,
            required_keys,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processor() {
        let rules = ValidationRules::new(
            0.0,
            100.0,
            vec!["temperature".to_string(), "humidity".to_string()]
        );
        
        let mut processor = DataProcessor::new(rules);
        
        assert!(processor.add_dataset(
            "temperature".to_string(),
            vec![20.5, 22.3, 18.7, 25.1]
        ).is_ok());
        
        assert!(processor.add_dataset(
            "humidity".to_string(),
            vec![45.2, 50.1, 48.7, 52.3]
        ).is_ok());
        
        let stats = processor.calculate_statistics();
        assert_eq!(stats.len(), 2);
        
        processor.normalize_data();
        
        let summary = processor.get_data_summary();
        assert_eq!(summary.dataset_count, 2);
        assert_eq!(summary.total_values, 8);
    }

    #[test]
    fn test_validation() {
        let rules = ValidationRules::new(
            0.0,
            100.0,
            vec!["valid_key".to_string()]
        );
        
        let mut processor = DataProcessor::new(rules);
        
        assert!(processor.add_dataset(
            "invalid_key".to_string(),
            vec![50.0]
        ).is_err());
        
        assert!(processor.add_dataset(
            "valid_key".to_string(),
            vec![150.0]
        ).is_err());
    }
}