use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, PartialEq)]
pub enum ValidationError {
    InvalidId,
    InvalidTimestamp,
    EmptyValues,
    InvalidMetadata,
}

impl DataRecord {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if self.timestamp <= 0 {
            return Err(ValidationError::InvalidTimestamp);
        }
        
        if self.values.is_empty() {
            return Err(ValidationError::EmptyValues);
        }
        
        if self.metadata.contains_key("") {
            return Err(ValidationError::InvalidMetadata);
        }
        
        Ok(())
    }
    
    pub fn transform(&mut self, factor: f64) -> &Self {
        for value in self.values.iter_mut() {
            *value *= factor;
        }
        self
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

pub fn process_records(records: &mut [DataRecord], factor: f64) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let mut processed = Vec::with_capacity(records.len());
    
    for record in records.iter_mut() {
        record.validate()?;
        let mut transformed = record.clone();
        transformed.transform(factor);
        processed.push(transformed);
    }
    
    Ok(processed)
}

pub fn filter_records(records: &[DataRecord], threshold: f64) -> Vec<&DataRecord> {
    records
        .iter()
        .filter(|record| {
            let (mean, _, _) = record.calculate_statistics();
            mean >= threshold
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validation() {
        let valid_record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::from([("key".to_string(), "value".to_string())]),
        };
        
        assert!(valid_record.validate().is_ok());
        
        let invalid_record = DataRecord {
            id: 0,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };
        
        assert_eq!(invalid_record.validate(), Err(ValidationError::InvalidId));
    }
    
    #[test]
    fn test_transform() {
        let mut record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };
        
        record.transform(2.0);
        assert_eq!(record.values, vec![2.0, 4.0, 6.0]);
    }
    
    #[test]
    fn test_statistics() {
        let record = DataRecord {
            id: 1,
            timestamp: 1234567890,
            values: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            metadata: HashMap::new(),
        };
        
        let (mean, variance, std_dev) = record.calculate_statistics();
        
        assert_eq!(mean, 3.0);
        assert_eq!(variance, 2.0);
        assert_eq!(std_dev, 2.0_f64.sqrt());
    }
}use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn process_data_file(path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut rdr = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in rdr.deserialize() {
        let record: Record = result?;
        if record.value >= 0.0 && record.category.len() <= 20 {
            records.push(record);
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, usize) {
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len();
    let avg = if count > 0 { sum / count as f64 } else { 0.0 };
    
    let max_value = records.iter()
        .map(|r| r.value)
        .fold(f64::NEG_INFINITY, |a, b| a.max(b));
    
    (avg, max_value, count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_process_valid_data() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,Test1,10.5,CategoryA").unwrap();
        writeln!(temp_file, "2,Test2,20.3,CategoryB").unwrap();
        
        let records = process_data_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "Test1");
    }

    #[test]
    fn test_calculate_stats() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, category: "X".to_string() },
            Record { id: 2, name: "B".to_string(), value: 20.0, category: "Y".to_string() },
        ];
        
        let (avg, max, count) = calculate_statistics(&records);
        assert_eq!(avg, 15.0);
        assert_eq!(max, 20.0);
        assert_eq!(count, 2);
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
    validation_rules: Vec<ValidationRule>,
}

pub struct ValidationRule {
    field_name: String,
    min_value: f64,
    max_value: f64,
    required: bool,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
            validation_rules: Vec::new(),
        }
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }

    pub fn process_dataset(&mut self, dataset_name: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Dataset cannot be empty".to_string());
        }

        self.validate_data(data)?;

        let processed_data: Vec<f64> = data
            .iter()
            .map(|&value| self.apply_transformations(value))
            .collect();

        self.cache.insert(dataset_name.to_string(), processed_data.clone());

        Ok(processed_data)
    }

    fn validate_data(&self, data: &[f64]) -> Result<(), String> {
        for rule in &self.validation_rules {
            if rule.required && data.is_empty() {
                return Err(format!("Field '{}' is required but missing", rule.field_name));
            }

            for &value in data {
                if value < rule.min_value || value > rule.max_value {
                    return Err(format!(
                        "Value {} for field '{}' is outside allowed range [{}, {}]",
                        value, rule.field_name, rule.min_value, rule.max_value
                    ));
                }
            }
        }
        Ok(())
    }

    fn apply_transformations(&self, value: f64) -> f64 {
        value.log10().abs()
    }

    pub fn get_cached_data(&self, dataset_name: &str) -> Option<&Vec<f64>> {
        self.cache.get(dataset_name)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn get_statistics(&self, dataset_name: &str) -> Option<DatasetStatistics> {
        self.cache.get(dataset_name).map(|data| {
            let sum: f64 = data.iter().sum();
            let count = data.len() as f64;
            let mean = sum / count;

            let variance: f64 = data.iter()
                .map(|&value| (value - mean).powi(2))
                .sum::<f64>() / count;

            DatasetStatistics {
                count: data.len(),
                mean,
                variance,
                std_dev: variance.sqrt(),
            }
        })
    }
}

pub struct DatasetStatistics {
    pub count: usize,
    pub mean: f64,
    pub variance: f64,
    pub std_dev: f64,
}

impl ValidationRule {
    pub fn new(field_name: &str, min_value: f64, max_value: f64, required: bool) -> Self {
        ValidationRule {
            field_name: field_name.to_string(),
            min_value,
            max_value,
            required,
        }
    }
}