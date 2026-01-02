
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

    pub fn calculate_statistics(&self, data: &[Vec<String>], column_index: usize) -> Result<(f64, f64), Box<dyn Error>> {
        if data.is_empty() {
            return Err("No data available for statistics".into());
        }

        let mut values = Vec::new();
        for record in data {
            if column_index >= record.len() {
                return Err(format!("Column index {} out of bounds", column_index).into());
            }
            
            match record[column_index].parse::<f64>() {
                Ok(value) => values.push(value),
                Err(_) => return Err(format!("Invalid numeric value at column {}", column_index).into()),
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
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000").unwrap();
        writeln!(temp_file, "Bob,25,45000").unwrap();
        
        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "30", "50000"]);
    }

    #[test]
    fn test_calculate_statistics() {
        let data = vec![
            vec!["10.5".to_string(), "20.0".to_string()],
            vec!["15.5".to_string(), "25.0".to_string()],
            vec!["12.0".to_string(), "30.0".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let (mean, std_dev) = processor.calculate_statistics(&data, 0).unwrap();
        
        assert!((mean - 12.666).abs() < 0.001);
        assert!((std_dev - 2.054).abs() < 0.001);
    }

    #[test]
    fn test_validate_record() {
        let processor = DataProcessor::new(',', false);
        
        let valid_record = vec!["field1".to_string(), "field2".to_string()];
        assert!(processor.validate_record(&valid_record));
        
        let invalid_record = vec!["".to_string(), "field2".to_string()];
        assert!(!processor.validate_record(&invalid_record));
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub timestamp: i64,
}

#[derive(Debug)]
pub enum ProcessingError {
    InvalidValue(f64),
    InvalidTimestamp(i64),
    MissingField(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidValue(v) => write!(f, "Invalid value: {}", v),
            ProcessingError::InvalidTimestamp(t) => write!(f, "Invalid timestamp: {}", t),
            ProcessingError::MissingField(field) => write!(f, "Missing field: {}", field),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Self {
        DataProcessor { threshold }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.value.is_nan() || record.value.is_infinite() {
            return Err(ProcessingError::InvalidValue(record.value));
        }

        if record.timestamp < 0 {
            return Err(ProcessingError::InvalidTimestamp(record.timestamp));
        }

        Ok(())
    }

    pub fn process_record(&self, record: &DataRecord) -> Result<DataRecord, ProcessingError> {
        self.validate_record(record)?;

        let processed_value = if record.value > self.threshold {
            record.value * 0.9
        } else {
            record.value * 1.1
        };

        Ok(DataRecord {
            id: record.id,
            value: processed_value,
            timestamp: record.timestamp,
        })
    }

    pub fn batch_process(
        &self,
        records: Vec<DataRecord>,
    ) -> (Vec<DataRecord>, Vec<ProcessingError>) {
        let mut processed = Vec::new();
        let mut errors = Vec::new();

        for record in records {
            match self.process_record(&record) {
                Ok(processed_record) => processed.push(processed_record),
                Err(error) => errors.push(error),
            }
        }

        (processed, errors)
    }
}

pub fn calculate_statistics(records: &[DataRecord]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = sum / count;

    let variance: f64 = records
        .iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>()
        / count;

    let std_dev = variance.sqrt();

    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_record() {
        let processor = DataProcessor::new(100.0);
        let valid_record = DataRecord {
            id: 1,
            value: 50.0,
            timestamp: 1625097600,
        };
        assert!(processor.validate_record(&valid_record).is_ok());

        let invalid_value = DataRecord {
            id: 2,
            value: f64::NAN,
            timestamp: 1625097600,
        };
        assert!(processor.validate_record(&invalid_value).is_err());

        let invalid_timestamp = DataRecord {
            id: 3,
            value: 50.0,
            timestamp: -1,
        };
        assert!(processor.validate_record(&invalid_timestamp).is_err());
    }

    #[test]
    fn test_process_record() {
        let processor = DataProcessor::new(100.0);
        let record = DataRecord {
            id: 1,
            value: 120.0,
            timestamp: 1625097600,
        };
        let processed = processor.process_record(&record).unwrap();
        assert_eq!(processed.value, 108.0);

        let record2 = DataRecord {
            id: 2,
            value: 80.0,
            timestamp: 1625097600,
        };
        let processed2 = processor.process_record(&record2).unwrap();
        assert_eq!(processed2.value, 88.0);
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            DataRecord {
                id: 1,
                value: 10.0,
                timestamp: 1625097600,
            },
            DataRecord {
                id: 2,
                value: 20.0,
                timestamp: 1625097600,
            },
            DataRecord {
                id: 3,
                value: 30.0,
                timestamp: 1625097600,
            },
        ];
        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

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
            if parts.len() != 3 {
                continue;
            }

            let id = match parts[0].parse::<u32>() {
                Ok(id) => id,
                Err(_) => continue,
            };

            let value = match parts[1].parse::<f64>() {
                Ok(value) => value,
                Err(_) => continue,
            };

            let category = parts[2].to_string();

            if !Self::validate_record(id, value, &category) {
                continue;
            }

            self.records.push(DataRecord { id, value, category });
            count += 1;
        }

        Ok(count)
    }

    fn validate_record(id: u32, value: f64, category: &str) -> bool {
        id > 0 && value >= 0.0 && !category.is_empty()
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

    pub fn record_count(&self) -> usize {
        self.records.len()
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
        assert_eq!(processor.record_count(), 0);

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,TypeA").unwrap();
        writeln!(temp_file, "2,20.3,TypeB").unwrap();
        writeln!(temp_file, "3,15.7,TypeA").unwrap();

        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);
        assert_eq!(processor.record_count(), 3);

        let avg = processor.calculate_average().unwrap();
        assert!((avg - 15.5).abs() < 0.01);

        let type_a_records = processor.filter_by_category("TypeA");
        assert_eq!(type_a_records.len(), 2);

        let stats = processor.get_statistics();
        assert!((stats.0 - 10.5).abs() < 0.01);
        assert!((stats.1 - 20.3).abs() < 0.01);
    }

    #[test]
    fn test_invalid_data_skipping() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,TypeA").unwrap();
        writeln!(temp_file, "0,20.3,TypeB").unwrap();
        writeln!(temp_file, "3,-5.0,TypeA").unwrap();
        writeln!(temp_file, "4,15.7,").unwrap();

        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 1);
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
pub enum ProcessingError {
    InvalidData(String),
    TransformationFailed(String),
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ProcessingError::TransformationFailed(msg) => write!(f, "Transformation failed: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    category_stats: HashMap<String, CategoryStatistics>,
}

#[derive(Debug, Clone)]
pub struct CategoryStatistics {
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

    fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        if record.name.trim().is_empty() {
            return Err(ProcessingError::ValidationError(
                "Record name cannot be empty".to_string(),
            ));
        }

        if record.value < 0.0 {
            return Err(ProcessingError::ValidationError(
                "Record value cannot be negative".to_string(),
            ));
        }

        if record.category.trim().is_empty() {
            return Err(ProcessingError::ValidationError(
                "Record category cannot be empty".to_string(),
            ));
        }

        Ok(())
    }

    fn update_category_stats(&mut self, record: &DataRecord) {
        let stats = self.category_stats
            .entry(record.category.clone())
            .or_insert(CategoryStatistics {
                category: record.category.clone(),
                count: 0,
                total_value: 0.0,
                average_value: 0.0,
            });

        stats.count += 1;
        stats.total_value += record.value;
        stats.average_value = stats.total_value / stats.count as f64;
    }

    pub fn get_category_statistics(&self, category: &str) -> Option<&CategoryStatistics> {
        self.category_stats.get(category)
    }

    pub fn transform_values<F>(&mut self, transform_fn: F) -> Result<(), ProcessingError>
    where
        F: Fn(f64) -> Result<f64, String>,
    {
        let mut transformed_records = Vec::new();

        for record in &self.records {
            match transform_fn(record.value) {
                Ok(transformed_value) => {
                    let transformed_record = DataRecord {
                        value: transformed_value,
                        ..record.clone()
                    };
                    transformed_records.push(transformed_record);
                }
                Err(err) => {
                    return Err(ProcessingError::TransformationFailed(format!(
                        "Failed to transform value {} for record {}: {}",
                        record.value, record.id, err
                    )));
                }
            }
        }

        self.records = transformed_records;
        self.recalculate_statistics();
        Ok(())
    }

    fn recalculate_statistics(&mut self) {
        self.category_stats.clear();
        for record in &self.records {
            self.update_category_stats(record);
        }
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn get_total_records(&self) -> usize {
        self.records.len()
    }

    pub fn get_average_value(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let total: f64 = self.records.iter().map(|r| r.value).sum();
        Some(total / self.records.len() as f64)
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
    fn test_add_valid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test Record".to_string(),
            value: 42.5,
            category: "Test".to_string(),
        };

        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.get_total_records(), 1);
    }

    #[test]
    fn test_add_invalid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "".to_string(),
            value: 42.5,
            category: "Test".to_string(),
        };

        assert!(processor.add_record(record).is_err());
        assert_eq!(processor.get_total_records(), 0);
    }

    #[test]
    fn test_category_statistics() {
        let mut processor = DataProcessor::new();
        
        let record1 = DataRecord {
            id: 1,
            name: "Record 1".to_string(),
            value: 10.0,
            category: "CategoryA".to_string(),
        };

        let record2 = DataRecord {
            id: 2,
            name: "Record 2".to_string(),
            value: 20.0,
            category: "CategoryA".to_string(),
        };

        processor.add_record(record1).unwrap();
        processor.add_record(record2).unwrap();

        let stats = processor.get_category_statistics("CategoryA").unwrap();
        assert_eq!(stats.count, 2);
        assert_eq!(stats.total_value, 30.0);
        assert_eq!(stats.average_value, 15.0);
    }

    #[test]
    fn test_value_transformation() {
        let mut processor = DataProcessor::new();
        
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 10.0,
            category: "Test".to_string(),
        };

        processor.add_record(record).unwrap();

        let transform = |x: f64| -> Result<f64, String> {
            if x < 0.0 {
                Err("Negative value".to_string())
            } else {
                Ok(x * 2.0)
            }
        };

        assert!(processor.transform_values(transform).is_ok());
        assert_eq!(processor.records[0].value, 20.0);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

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
            
            let category = parts[2].trim().to_string();
            
            if !self.validate_record(&category, value) {
                continue;
            }
            
            self.records.push(DataRecord { id, value, category });
            count += 1;
        }
        
        Ok(count)
    }
    
    fn validate_record(&self, category: &str, value: f64) -> bool {
        !category.is_empty() && value >= 0.0 && value <= 1000.0
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
            .filter(|r| r.category == category)
            .collect()
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
    
    pub fn record_count(&self) -> usize {
        self.records.len()
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
        assert_eq!(processor.record_count(), 0);
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,100.5,TypeA").unwrap();
        writeln!(temp_file, "2,200.3,TypeB").unwrap();
        writeln!(temp_file, "3,300.7,TypeA").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(processor.record_count(), 3);
        
        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 200.5).abs() < 0.1);
        
        let type_a_records = processor.filter_by_category("TypeA");
        assert_eq!(type_a_records.len(), 2);
        
        let stats = processor.get_statistics();
        assert!((stats.0 - 100.5).abs() < 0.1);
        assert!((stats.1 - 300.7).abs() < 0.1);
    }
    
    #[test]
    fn test_invalid_data_skipping() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,invalid,TypeA").unwrap();
        writeln!(temp_file, "2,-10.0,TypeB").unwrap();
        writeln!(temp_file, "3,1500.0,TypeC").unwrap();
        writeln!(temp_file, "4,250.5,TypeA").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }
}