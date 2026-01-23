
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    name: String,
    value: f64,
    tags: Vec<String>,
}

#[derive(Debug)]
pub enum DataError {
    InvalidId,
    InvalidName,
    InvalidValue,
    EmptyTags,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "ID must be greater than zero"),
            DataError::InvalidName => write!(f, "Name cannot be empty"),
            DataError::InvalidValue => write!(f, "Value must be positive"),
            DataError::EmptyTags => write!(f, "Record must have at least one tag"),
        }
    }
}

impl Error for DataError {}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, tags: Vec<String>) -> Result<Self, DataError> {
        if id == 0 {
            return Err(DataError::InvalidId);
        }
        if name.trim().is_empty() {
            return Err(DataError::InvalidName);
        }
        if value <= 0.0 {
            return Err(DataError::InvalidValue);
        }
        if tags.is_empty() {
            return Err(DataError::EmptyTags);
        }

        Ok(Self {
            id,
            name,
            value,
            tags,
        })
    }

    pub fn transform(&self, multiplier: f64) -> Self {
        Self {
            id: self.id,
            name: self.name.clone(),
            value: self.value * multiplier,
            tags: self.tags.clone(),
        }
    }

    pub fn summary(&self) -> String {
        format!(
            "Record {}: {} (value: {:.2}) with {} tags",
            self.id,
            self.name,
            self.value,
            self.tags.len()
        )
    }
}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        if self.records.contains_key(&record.id) {
            return Err(DataError::InvalidId);
        }
        self.records.insert(record.id, record);
        Ok(())
    }

    pub fn process_records(&mut self, multiplier: f64) -> Vec<DataRecord> {
        let mut processed = Vec::new();
        for record in self.records.values() {
            processed.push(record.transform(multiplier));
        }
        processed
    }

    pub fn get_statistics(&self) -> (usize, f64, f64) {
        let count = self.records.len();
        let total_value: f64 = self.records.values().map(|r| r.value).sum();
        let avg_value = if count > 0 {
            total_value / count as f64
        } else {
            0.0
        };
        (count, total_value, avg_value)
    }

    pub fn find_by_tag(&self, tag: &str) -> Vec<&DataRecord> {
        self.records
            .values()
            .filter(|record| record.tags.iter().any(|t| t == tag))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(
            1,
            "Test Record".to_string(),
            100.0,
            vec!["tag1".to_string(), "tag2".to_string()],
        );
        assert!(record.is_ok());
    }

    #[test]
    fn test_invalid_id() {
        let record = DataRecord::new(
            0,
            "Test".to_string(),
            50.0,
            vec!["tag".to_string()],
        );
        assert!(matches!(record, Err(DataError::InvalidId)));
    }

    #[test]
    fn test_data_processor_operations() {
        let mut processor = DataProcessor::new();
        let record = DataRecord::new(
            1,
            "Sample".to_string(),
            10.0,
            vec!["important".to_string()],
        ).unwrap();

        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.get_statistics().0, 1);
        
        let tagged = processor.find_by_tag("important");
        assert_eq!(tagged.len(), 1);
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

    pub fn process_dataset(&mut self, dataset_name: &str, data: Vec<f64>) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Dataset cannot be empty".to_string());
        }

        for rule in &self.validation_rules {
            if rule.required && data.iter().any(|&x| x.is_nan()) {
                return Err(format!("Field {} contains invalid values", rule.field_name));
            }
        }

        let processed_data: Vec<f64> = data
            .iter()
            .map(|&value| {
                let mut transformed = value;
                for rule in &self.validation_rules {
                    if value < rule.min_value {
                        transformed = rule.min_value;
                    } else if value > rule.max_value {
                        transformed = rule.max_value;
                    }
                }
                transformed
            })
            .collect();

        self.cache.insert(dataset_name.to_string(), processed_data.clone());
        Ok(processed_data)
    }

    pub fn get_cached_data(&self, dataset_name: &str) -> Option<&Vec<f64>> {
        self.cache.get(dataset_name)
    }

    pub fn calculate_statistics(&self, dataset_name: &str) -> Option<DatasetStats> {
        self.cache.get(dataset_name).map(|data| {
            let sum: f64 = data.iter().sum();
            let count = data.len() as f64;
            let mean = sum / count;
            
            let variance: f64 = data.iter()
                .map(|&value| (value - mean).powi(2))
                .sum::<f64>() / count;
            
            DatasetStats {
                mean,
                variance,
                min: *data.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
                max: *data.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
                count: data.len(),
            }
        })
    }
}

pub struct DatasetStats {
    pub mean: f64,
    pub variance: f64,
    pub min: f64,
    pub max: f64,
    pub count: usize,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(ValidationRule::new("temperature", -50.0, 100.0, true));
        
        let data = vec![25.0, 30.0, 105.0, -60.0, 75.0];
        let result = processor.process_dataset("weather", data);
        
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(processed, vec![25.0, 30.0, 100.0, -50.0, 75.0]);
    }

    #[test]
    fn test_empty_dataset() {
        let mut processor = DataProcessor::new();
        let result = processor.process_dataset("empty", vec![]);
        assert!(result.is_err());
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    data: Vec<f64>,
    frequency_map: HashMap<String, usize>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: Vec::new(),
            frequency_map: HashMap::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        for line in reader.lines().skip(1) {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            
            if parts.len() >= 2 {
                if let Ok(value) = parts[1].parse::<f64>() {
                    self.data.push(value);
                }
                
                let category = parts[0].to_string();
                *self.frequency_map.entry(category).or_insert(0) += 1;
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

    pub fn calculate_median(&mut self) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }
        
        self.data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mid = self.data.len() / 2;
        
        if self.data.len() % 2 == 0 {
            Some((self.data[mid - 1] + self.data[mid]) / 2.0)
        } else {
            Some(self.data[mid])
        }
    }

    pub fn get_frequency_distribution(&self) -> &HashMap<String, usize> {
        &self.frequency_map
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<f64> {
        self.data.iter()
            .filter(|&&x| x > threshold)
            .copied()
            .collect()
    }

    pub fn data_summary(&self) -> String {
        let mean = self.calculate_mean().unwrap_or(0.0);
        let count = self.data.len();
        let unique_categories = self.frequency_map.len();
        
        format!(
            "Data Summary:\n  Total entries: {}\n  Unique categories: {}\n  Mean value: {:.2}",
            count, unique_categories, mean
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
        writeln!(temp_file, "category,value").unwrap();
        writeln!(temp_file, "A,10.5").unwrap();
        writeln!(temp_file, "B,20.3").unwrap();
        writeln!(temp_file, "A,15.7").unwrap();
        writeln!(temp_file, "C,8.9").unwrap();
        
        processor.load_from_csv(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.calculate_mean(), Some(13.85));
        assert_eq!(processor.data.len(), 4);
        assert_eq!(processor.frequency_map.get("A"), Some(&2));
    }
}use std::error::Error;
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
        DataRecord {
            id,
            value,
            category,
            timestamp,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.category.is_empty() 
            && self.value.is_finite() 
            && self.id > 0
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
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 42.5, "test".to_string(), "2024-01-01".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(0, f64::NAN, "".to_string(), "2024-01-01".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category,timestamp").unwrap();
        writeln!(temp_file, "1,10.5,alpha,2024-01-01").unwrap();
        writeln!(temp_file, "2,20.0,beta,2024-01-02").unwrap();
        writeln!(temp_file, "3,30.5,alpha,2024-01-03").unwrap();

        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);
        assert_eq!(processor.count_records(), 3);

        let alpha_records = processor.filter_by_category("alpha");
        assert_eq!(alpha_records.len(), 2);

        let stats = processor.get_statistics();
        assert_eq!(stats.0, 10.5);
        assert_eq!(stats.1, 30.5);
    }
}