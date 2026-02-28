
use std::collections::HashMap;

pub struct DataProcessor {
    validators: HashMap<String, Box<dyn Fn(&str) -> bool>>,
    transformers: HashMap<String, Box<dyn Fn(String) -> String>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        let mut processor = DataProcessor {
            validators: HashMap::new(),
            transformers: HashMap::new(),
        };
        
        processor.register_default_validators();
        processor.register_default_transformers();
        
        processor
    }
    
    fn register_default_validators(&mut self) {
        self.validators.insert(
            "email".to_string(),
            Box::new(|input: &str| {
                input.contains('@') && input.contains('.') && input.len() > 5
            })
        );
        
        self.validators.insert(
            "numeric".to_string(),
            Box::new(|input: &str| {
                input.chars().all(|c| c.is_ascii_digit())
            })
        );
    }
    
    fn register_default_transformers(&mut self) {
        self.transformers.insert(
            "uppercase".to_string(),
            Box::new(|input: String| input.to_uppercase())
        );
        
        self.transformers.insert(
            "trim".to_string(),
            Box::new(|input: String| input.trim().to_string())
        );
    }
    
    pub fn validate(&self, validator_name: &str, input: &str) -> bool {
        match self.validators.get(validator_name) {
            Some(validator) => validator(input),
            None => false,
        }
    }
    
    pub fn transform(&self, transformer_name: &str, input: String) -> String {
        match self.transformers.get(transformer_name) {
            Some(transformer) => transformer(input),
            None => input,
        }
    }
    
    pub fn process_pipeline(&self, input: String, operations: Vec<(&str, &str)>) -> Result<String, String> {
        let mut result = input;
        
        for (op_type, op_name) in operations {
            match op_type {
                "validate" => {
                    if !self.validate(op_name, &result) {
                        return Err(format!("Validation failed for '{}'", op_name));
                    }
                }
                "transform" => {
                    result = self.transform(op_name, result);
                }
                _ => return Err(format!("Unknown operation type: {}", op_type)),
            }
        }
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_email_validation() {
        let processor = DataProcessor::new();
        assert!(processor.validate("email", "test@example.com"));
        assert!(!processor.validate("email", "invalid-email"));
    }
    
    #[test]
    fn test_numeric_validation() {
        let processor = DataProcessor::new();
        assert!(processor.validate("numeric", "12345"));
        assert!(!processor.validate("numeric", "123abc"));
    }
    
    #[test]
    fn test_uppercase_transformation() {
        let processor = DataProcessor::new();
        let result = processor.transform("uppercase", "hello".to_string());
        assert_eq!(result, "HELLO");
    }
    
    #[test]
    fn test_processing_pipeline() {
        let processor = DataProcessor::new();
        let operations = vec![
            ("validate", "email"),
            ("transform", "uppercase"),
        ];
        
        let result = processor.process_pipeline("test@example.com".to_string(), operations);
        assert_eq!(result.unwrap(), "TEST@EXAMPLE.COM");
    }
}use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let mut rdr = Reader::from_path(path)?;
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        Ok(())
    }

    pub fn validate_records(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.value >= 0.0 && !r.name.is_empty())
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records = self.validate_records();
        if valid_records.is_empty() {
            return None;
        }
        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&Record>> {
        let mut groups = std::collections::HashMap::new();
        for record in &self.records {
            groups
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record);
        }
        groups
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_data_processing() {
        let csv_data = "id,name,value,category\n1,ItemA,10.5,Alpha\n2,ItemB,15.0,Beta\n3,ItemC,-5.0,Alpha";
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_data).unwrap();

        let mut processor = DataProcessor::new();
        processor.load_from_csv(temp_file.path()).unwrap();

        let valid = processor.validate_records();
        assert_eq!(valid.len(), 2);

        let avg = processor.calculate_average();
        assert_eq!(avg, Some(12.75));

        let groups = processor.group_by_category();
        assert_eq!(groups.get("Alpha").unwrap().len(), 1);
        assert_eq!(groups.get("Beta").unwrap().len(), 1);
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
    category_stats: HashMap<String, CategoryStats>,
}

#[derive(Debug, Clone)]
pub struct CategoryStats {
    pub total_records: usize,
    pub total_value: f64,
    pub average_value: f64,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: HashMap::new(),
            category_stats: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        if record.id == 0 {
            return Err(DataError::InvalidId);
        }

        if record.value < 0.0 || record.value.is_nan() || record.value.is_infinite() {
            return Err(DataError::InvalidValue);
        }

        if record.name.is_empty() || record.category.is_empty() {
            return Err(DataError::MissingField);
        }

        if self.records.contains_key(&record.id) {
            return Err(DataError::DuplicateRecord);
        }

        self.records.insert(record.id, record.clone());
        self.update_category_stats(&record);

        Ok(())
    }

    fn update_category_stats(&mut self, record: &DataRecord) {
        let stats = self.category_stats
            .entry(record.category.clone())
            .or_insert(CategoryStats {
                total_records: 0,
                total_value: 0.0,
                average_value: 0.0,
            });

        stats.total_records += 1;
        stats.total_value += record.value;
        stats.average_value = stats.total_value / stats.total_records as f64;
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn get_category_stats(&self, category: &str) -> Option<&CategoryStats> {
        self.category_stats.get(category)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .values()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.values().map(|record| record.value).sum()
    }

    pub fn get_all_categories(&self) -> Vec<String> {
        self.category_stats.keys().cloned().collect()
    }

    pub fn remove_record(&mut self, id: u32) -> Option<DataRecord> {
        if let Some(record) = self.records.remove(&id) {
            self.recalculate_category_stats();
            Some(record)
        } else {
            None
        }
    }

    fn recalculate_category_stats(&mut self) {
        self.category_stats.clear();
        
        for record in self.records.values() {
            self.update_category_stats(record);
        }
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
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
            value: 100.0,
            category: "Test".to_string(),
        };

        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.record_count(), 1);
    }

    #[test]
    fn test_add_invalid_record() {
        let mut processor = DataProcessor::new();
        let record = DataRecord {
            id: 0,
            name: "Invalid".to_string(),
            value: 50.0,
            category: "Test".to_string(),
        };

        assert!(processor.add_record(record).is_err());
        assert_eq!(processor.record_count(), 0);
    }

    #[test]
    fn test_duplicate_record() {
        let mut processor = DataProcessor::new();
        let record1 = DataRecord {
            id: 1,
            name: "Record 1".to_string(),
            value: 100.0,
            category: "Test".to_string(),
        };

        let record2 = DataRecord {
            id: 1,
            name: "Record 2".to_string(),
            value: 200.0,
            category: "Test".to_string(),
        };

        assert!(processor.add_record(record1).is_ok());
        assert!(processor.add_record(record2).is_err());
        assert_eq!(processor.record_count(), 1);
    }

    #[test]
    fn test_category_stats() {
        let mut processor = DataProcessor::new();
        
        let records = vec![
            DataRecord {
                id: 1,
                name: "Record 1".to_string(),
                value: 100.0,
                category: "CategoryA".to_string(),
            },
            DataRecord {
                id: 2,
                name: "Record 2".to_string(),
                value: 200.0,
                category: "CategoryA".to_string(),
            },
            DataRecord {
                id: 3,
                name: "Record 3".to_string(),
                value: 300.0,
                category: "CategoryB".to_string(),
            },
        ];

        for record in records {
            processor.add_record(record).unwrap();
        }

        let stats_a = processor.get_category_stats("CategoryA").unwrap();
        assert_eq!(stats_a.total_records, 2);
        assert_eq!(stats_a.total_value, 300.0);
        assert_eq!(stats_a.average_value, 150.0);

        let stats_b = processor.get_category_stats("CategoryB").unwrap();
        assert_eq!(stats_b.total_records, 1);
        assert_eq!(stats_b.total_value, 300.0);
        assert_eq!(stats_b.average_value, 300.0);
    }

    #[test]
    fn test_filter_records() {
        let mut processor = DataProcessor::new();
        
        let records = vec![
            DataRecord {
                id: 1,
                name: "Record 1".to_string(),
                value: 100.0,
                category: "CategoryA".to_string(),
            },
            DataRecord {
                id: 2,
                name: "Record 2".to_string(),
                value: 200.0,
                category: "CategoryB".to_string(),
            },
            DataRecord {
                id: 3,
                name: "Record 3".to_string(),
                value: 300.0,
                category: "CategoryA".to_string(),
            },
        ];

        for record in records {
            processor.add_record(record).unwrap();
        }

        let category_a_records = processor.filter_by_category("CategoryA");
        assert_eq!(category_a_records.len(), 2);
        
        let category_b_records = processor.filter_by_category("CategoryB");
        assert_eq!(category_b_records.len(), 1);
    }

    #[test]
    fn test_remove_record() {
        let mut processor = DataProcessor::new();
        
        let record = DataRecord {
            id: 1,
            name: "Test Record".to_string(),
            value: 100.0,
            category: "Test".to_string(),
        };

        processor.add_record(record.clone()).unwrap();
        assert_eq!(processor.record_count(), 1);
        
        let removed = processor.remove_record(1);
        assert!(removed.is_some());
        assert_eq!(processor.record_count(), 0);
        assert!(processor.is_empty());
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
    pub valid: bool,
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    total_value: f64,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            total_value: 0.0,
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
            let valid = parts[3].to_lowercase() == "true";

            let record = DataRecord {
                id,
                value,
                category,
                valid,
            };

            if record.valid {
                self.total_value += record.value;
            }

            self.records.push(record);
            count += 1;
        }

        Ok(count)
    }

    pub fn get_average_value(&self) -> Option<f64> {
        let valid_count = self.records.iter().filter(|r| r.valid).count();
        if valid_count > 0 {
            Some(self.total_value / valid_count as f64)
        } else {
            None
        }
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category && record.valid)
            .collect()
    }

    pub fn get_statistics(&self) -> (usize, usize, f64) {
        let total = self.records.len();
        let valid = self.records.iter().filter(|r| r.valid).count();
        let avg = self.get_average_value().unwrap_or(0.0);
        (total, valid, avg)
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
        writeln!(temp_file, "id,value,category,valid").unwrap();
        writeln!(temp_file, "1,10.5,alpha,true").unwrap();
        writeln!(temp_file, "2,20.0,beta,true").unwrap();
        writeln!(temp_file, "3,invalid,gamma,false").unwrap();
        writeln!(temp_file, "4,15.75,alpha,true").unwrap();

        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);

        let stats = processor.get_statistics();
        assert_eq!(stats.0, 3);
        assert_eq!(stats.1, 3);

        let alpha_records = processor.filter_by_category("alpha");
        assert_eq!(alpha_records.len(), 2);

        let avg = processor.get_average_value().unwrap();
        assert!((avg - 15.416).abs() < 0.001);
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
pub enum DataError {
    InvalidId,
    InvalidName,
    InvalidValue,
    InvalidCategory,
    DuplicateRecord,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "Invalid record ID"),
            DataError::InvalidName => write!(f, "Invalid record name"),
            DataError::InvalidValue => write!(f, "Invalid value"),
            DataError::InvalidCategory => write!(f, "Invalid category"),
            DataError::DuplicateRecord => write!(f, "Duplicate record found"),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
    category_stats: HashMap<String, CategoryStats>,
}

#[derive(Debug, Clone)]
pub struct CategoryStats {
    count: usize,
    total_value: f64,
    average_value: f64,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: HashMap::new(),
            category_stats: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        if record.id == 0 {
            return Err(DataError::InvalidId);
        }

        if record.name.trim().is_empty() {
            return Err(DataError::InvalidName);
        }

        if record.value <= 0.0 || record.value.is_nan() || record.value.is_infinite() {
            return Err(DataError::InvalidValue);
        }

        if record.category.trim().is_empty() {
            return Err(DataError::InvalidCategory);
        }

        if self.records.contains_key(&record.id) {
            return Err(DataError::DuplicateRecord);
        }

        self.records.insert(record.id, record.clone());
        self.update_category_stats(&record);

        Ok(())
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn remove_record(&mut self, id: u32) -> Option<DataRecord> {
        let record = self.records.remove(&id);
        
        if let Some(ref rec) = record {
            self.recalculate_category_stats();
        }

        record
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.values().map(|r| r.value).sum()
    }

    pub fn get_category_stats(&self, category: &str) -> Option<&CategoryStats> {
        self.category_stats.get(category)
    }

    pub fn transform_values<F>(&mut self, transform_fn: F) 
    where
        F: Fn(f64) -> f64,
    {
        for record in self.records.values_mut() {
            record.value = transform_fn(record.value);
        }
        self.recalculate_category_stats();
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .values()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn get_all_categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self.records
            .values()
            .map(|r| r.category.clone())
            .collect();
        
        categories.sort();
        categories.dedup();
        categories
    }

    fn update_category_stats(&mut self, record: &DataRecord) {
        let stats = self.category_stats
            .entry(record.category.clone())
            .or_insert(CategoryStats {
                count: 0,
                total_value: 0.0,
                average_value: 0.0,
            });

        stats.count += 1;
        stats.total_value += record.value;
        stats.average_value = stats.total_value / stats.count as f64;
    }

    fn recalculate_category_stats(&mut self) {
        self.category_stats.clear();
        
        for record in self.records.values() {
            self.update_category_stats(record);
        }
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
            value: 100.0,
            category: "Test".to_string(),
        };

        assert!(processor.add_record(record).is_ok());
        assert_eq!(processor.records.len(), 1);
    }

    #[test]
    fn test_add_invalid_record() {
        let mut processor = DataProcessor::new();
        let invalid_record = DataRecord {
            id: 0,
            name: "".to_string(),
            value: -1.0,
            category: "".to_string(),
        };

        assert!(processor.add_record(invalid_record).is_err());
    }

    #[test]
    fn test_duplicate_record() {
        let mut processor = DataProcessor::new();
        let record1 = DataRecord {
            id: 1,
            name: "Record 1".to_string(),
            value: 50.0,
            category: "Category A".to_string(),
        };

        let record2 = DataRecord {
            id: 1,
            name: "Record 2".to_string(),
            value: 75.0,
            category: "Category B".to_string(),
        };

        assert!(processor.add_record(record1).is_ok());
        assert!(processor.add_record(record2).is_err());
    }

    #[test]
    fn test_category_stats() {
        let mut processor = DataProcessor::new();
        
        let records = vec![
            DataRecord {
                id: 1,
                name: "Item 1".to_string(),
                value: 10.0,
                category: "Electronics".to_string(),
            },
            DataRecord {
                id: 2,
                name: "Item 2".to_string(),
                value: 20.0,
                category: "Electronics".to_string(),
            },
            DataRecord {
                id: 3,
                name: "Item 3".to_string(),
                value: 15.0,
                category: "Books".to_string(),
            },
        ];

        for record in records {
            processor.add_record(record).unwrap();
        }

        let stats = processor.get_category_stats("Electronics").unwrap();
        assert_eq!(stats.count, 2);
        assert_eq!(stats.total_value, 30.0);
        assert_eq!(stats.average_value, 15.0);
    }

    #[test]
    fn test_value_transformation() {
        let mut processor = DataProcessor::new();
        
        let record = DataRecord {
            id: 1,
            name: "Test Item".to_string(),
            value: 10.0,
            category: "Test".to_string(),
        };

        processor.add_record(record).unwrap();
        
        processor.transform_values(|x| x * 2.0);
        
        let updated_record = processor.get_record(1).unwrap();
        assert_eq!(updated_record.value, 20.0);
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
        let mut lines = reader.lines();

        if self.has_header {
            lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !fields.is_empty() && !fields.iter().all(|f| f.is_empty()) {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn validate_records(&self, records: &[Vec<String>], expected_columns: usize) -> Vec<usize> {
        let mut invalid_rows = Vec::new();
        
        for (index, record) in records.iter().enumerate() {
            if record.len() != expected_columns {
                invalid_rows.push(index);
            }
        }
        
        invalid_rows
    }

    pub fn extract_column(&self, records: &[Vec<String>], column_index: usize) -> Vec<String> {
        records
            .iter()
            .filter_map(|record| record.get(column_index).cloned())
            .collect()
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
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,30,New York").unwrap();
        writeln!(temp_file, "Alice,25,London").unwrap();
        
        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["John", "30", "New York"]);
    }

    #[test]
    fn test_validate_records() {
        let records = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string()],
            vec!["d".to_string(), "e".to_string(), "f".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let invalid = processor.validate_records(&records, 2);
        
        assert_eq!(invalid, vec![1, 2]);
    }

    #[test]
    fn test_extract_column() {
        let records = vec![
            vec!["apple".to_string(), "red".to_string()],
            vec!["banana".to_string(), "yellow".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let colors = processor.extract_column(&records, 1);
        
        assert_eq!(colors, vec!["red", "yellow"]);
    }
}