
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

    pub fn process_dataset(&mut self, key: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Empty dataset provided".to_string());
        }

        if let Some(cached) = self.cache.get(key) {
            return Ok(cached.clone());
        }

        let processed = Self::normalize_data(data)?;
        self.cache.insert(key.to_string(), processed.clone());
        Ok(processed)
    }

    fn normalize_data(data: &[f64]) -> Result<Vec<f64>, String> {
        let max_value = data
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        if max_value <= 0.0 {
            return Err("Invalid data range for normalization".to_string());
        }

        Ok(data
            .iter()
            .map(|&value| value / max_value)
            .collect())
    }

    pub fn calculate_statistics(&self, data: &[f64]) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if data.is_empty() {
            return stats;
        }

        let sum: f64 = data.iter().sum();
        let count = data.len() as f64;
        let mean = sum / count;

        let variance: f64 = data
            .iter()
            .map(|&value| (value - mean).powi(2))
            .sum::<f64>() / count;

        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("count".to_string(), count);
        stats.insert("sum".to_string(), sum);

        stats
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
    fn test_normalize_data() {
        let data = vec![1.0, 2.0, 3.0, 4.0];
        let result = DataProcessor::normalize_data(&data).unwrap();
        assert_eq!(result, vec![0.25, 0.5, 0.75, 1.0]);
    }

    #[test]
    fn test_empty_dataset() {
        let mut processor = DataProcessor::new();
        let result = processor.process_dataset("test", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_cache_functionality() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0];
        
        let first_result = processor.process_dataset("key1", &data).unwrap();
        let second_result = processor.process_dataset("key1", &data).unwrap();
        
        assert_eq!(first_result, second_result);
        assert_eq!(processor.cache_size(), 1);
    }

    #[test]
    fn test_statistics_calculation() {
        let processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0, 4.0];
        let stats = processor.calculate_statistics(&data);
        
        assert_eq!(stats.get("mean"), Some(&2.5));
        assert_eq!(stats.get("count"), Some(&4.0));
        assert_eq!(stats.get("sum"), Some(&10.0));
    }
}
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

    pub fn process_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (line_number, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line.is_empty() {
                continue;
            }

            if self.has_header && line_number == 0 {
                continue;
            }

            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if !self.validate_record(&fields) {
                return Err(format!("Invalid record at line {}", line_number + 1).into());
            }

            records.push(fields);
        }

        Ok(records)
    }

    fn validate_record(&self, fields: &[String]) -> bool {
        !fields.is_empty() && fields.iter().all(|field| !field.is_empty())
    }

    pub fn calculate_statistics(&self, records: &[Vec<String>], column_index: usize) -> Result<(f64, f64), Box<dyn Error>> {
        if records.is_empty() {
            return Err("No records to process".into());
        }

        let mut values = Vec::new();
        for record in records {
            if column_index >= record.len() {
                return Err(format!("Column index {} out of bounds", column_index).into());
            }

            if let Ok(value) = record[column_index].parse::<f64>() {
                values.push(value);
            } else {
                return Err(format!("Invalid numeric value at column {}", column_index).into());
            }
        }

        let sum: f64 = values.iter().sum();
        let mean = sum / values.len() as f64;
        
        let variance: f64 = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        
        let std_dev = variance.sqrt();

        Ok((mean, std_dev))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_file_with_header() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,score").unwrap();
        writeln!(temp_file, "Alice,25,95.5").unwrap();
        writeln!(temp_file, "Bob,30,88.0").unwrap();
        
        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "25", "95.5"]);
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            vec!["10.0".to_string(), "20.0".to_string()],
            vec!["20.0".to_string(), "30.0".to_string()],
            vec!["30.0".to_string(), "40.0".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let (mean, std_dev) = processor.calculate_statistics(&records, 0).unwrap();
        
        assert!((mean - 20.0).abs() < 0.001);
        assert!((std_dev - 8.164966).abs() < 0.001);
    }

    #[test]
    fn test_validate_record() {
        let processor = DataProcessor::new(',', false);
        
        let valid_record = vec!["data".to_string(), "value".to_string()];
        assert!(processor.validate_record(&valid_record));
        
        let invalid_record = vec!["".to_string(), "value".to_string()];
        assert!(!processor.validate_record(&invalid_record));
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
    InvalidCategory,
    DuplicateRecord,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "ID must be greater than 0"),
            DataError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            DataError::EmptyName => write!(f, "Name cannot be empty"),
            DataError::InvalidCategory => write!(f, "Category must be one of: A, B, C, D"),
            DataError::DuplicateRecord => write!(f, "Record with this ID already exists"),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
    category_stats: HashMap<String, (u32, f64)>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: HashMap::new(),
            category_stats: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        self.validate_record(&record)?;
        
        if self.records.contains_key(&record.id) {
            return Err(DataError::DuplicateRecord);
        }

        self.update_category_stats(&record, true);
        self.records.insert(record.id, record);
        
        Ok(())
    }

    pub fn remove_record(&mut self, id: u32) -> Option<DataRecord> {
        if let Some(record) = self.records.remove(&id) {
            self.update_category_stats(&record, false);
            Some(record)
        } else {
            None
        }
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn get_records_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .values()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn calculate_average(&self, category: &str) -> Option<f64> {
        self.category_stats
            .get(category)
            .map(|(count, sum)| sum / count as f64)
    }

    pub fn transform_values<F>(&mut self, transform_fn: F) 
    where
        F: Fn(f64) -> f64,
    {
        for record in self.records.values_mut() {
            let old_value = record.value;
            record.value = transform_fn(record.value);
            
            if let Some((count, sum)) = self.category_stats.get_mut(&record.category) {
                *sum = *sum - old_value + record.value;
            }
        }
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), DataError> {
        if record.id == 0 {
            return Err(DataError::InvalidId);
        }
        
        if record.name.trim().is_empty() {
            return Err(DataError::EmptyName);
        }
        
        if !(0.0..=1000.0).contains(&record.value) {
            return Err(DataError::InvalidValue);
        }
        
        let valid_categories = ["A", "B", "C", "D"];
        if !valid_categories.contains(&record.category.as_str()) {
            return Err(DataError::InvalidCategory);
        }
        
        Ok(())
    }

    fn update_category_stats(&mut self, record: &DataRecord, add: bool) {
        let entry = self.category_stats
            .entry(record.category.clone())
            .or_insert((0, 0.0));
        
        if add {
            entry.0 += 1;
            entry.1 += record.value;
        } else {
            entry.0 -= 1;
            entry.1 -= record.value;
            
            if entry.0 == 0 {
                self.category_stats.remove(&record.category);
            }
        }
    }

    pub fn total_records(&self) -> usize {
        self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
        self.category_stats.clear();
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
            category: "A".to_string(),
        };
        
        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.total_records(), 1);
    }

    #[test]
    fn test_add_invalid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 0,
            name: "".to_string(),
            value: -10.0,
            category: "X".to_string(),
        };
        
        assert!(processor.add_record(record).is_err());
    }

    #[test]
    fn test_remove_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 50.0,
            category: "B".to_string(),
        };
        
        processor.add_record(record).unwrap();
        assert_eq!(processor.total_records(), 1);
        
        let removed = processor.remove_record(1);
        assert!(removed.is_some());
        assert_eq!(processor.total_records(), 0);
    }

    #[test]
    fn test_transform_values() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 10.0,
            category: "C".to_string(),
        };
        
        processor.add_record(record).unwrap();
        processor.transform_values(|x| x * 2.0);
        
        let updated = processor.get_record(1).unwrap();
        assert_eq!(updated.value, 20.0);
    }
}