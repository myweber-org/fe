
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u64, timestamp: i64) -> Self {
        Self {
            id,
            timestamp,
            values: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_value(&mut self, value: f64) -> &mut Self {
        self.values.push(value);
        self
    }

    pub fn add_metadata(&mut self, key: String, value: String) -> &mut Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0 {
            return Err("ID cannot be zero".to_string());
        }
        if self.timestamp < 0 {
            return Err("Timestamp cannot be negative".to_string());
        }
        if self.values.is_empty() {
            return Err("Values cannot be empty".to_string());
        }
        Ok(())
    }

    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if self.values.is_empty() {
            return stats;
        }

        let sum: f64 = self.values.iter().sum();
        let count = self.values.len() as f64;
        let mean = sum / count;

        let variance: f64 = self.values
            .iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>() / count;

        let min = self.values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        stats.insert("mean".to_string(), mean);
        stats.insert("variance".to_string(), variance);
        stats.insert("min".to_string(), min);
        stats.insert("max".to_string(), max);
        stats.insert("count".to_string(), count);
        stats.insert("sum".to_string(), sum);

        stats
    }
}

pub fn process_records(records: &[DataRecord]) -> Vec<HashMap<String, f64>> {
    records
        .iter()
        .filter(|record| record.validate().is_ok())
        .map(|record| record.calculate_statistics())
        .collect()
}

pub fn filter_by_metadata(
    records: &[DataRecord],
    key: &str,
    value: &str,
) -> Vec<DataRecord> {
    records
        .iter()
        .filter(|record| {
            record.metadata
                .get(key)
                .map_or(false, |v| v == value)
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 1234567890);
        assert_eq!(record.id, 1);
        assert_eq!(record.timestamp, 1234567890);
        assert!(record.values.is_empty());
        assert!(record.metadata.is_empty());
    }

    #[test]
    fn test_add_value_and_metadata() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value(42.0)
              .add_value(24.0)
              .add_metadata("source".to_string(), "test".to_string());

        assert_eq!(record.values.len(), 2);
        assert_eq!(record.values[0], 42.0);
        assert_eq!(record.values[1], 24.0);
        assert_eq!(record.metadata.get("source"), Some(&"test".to_string()));
    }

    #[test]
    fn test_validation() {
        let valid_record = DataRecord::new(1, 1234567890);
        valid_record.add_value(10.0);
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(0, 1234567890);
        invalid_record.add_value(10.0);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_calculate_statistics() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value(10.0).add_value(20.0).add_value(30.0);

        let stats = record.calculate_statistics();
        assert_eq!(stats.get("mean"), Some(&20.0));
        assert_eq!(stats.get("min"), Some(&10.0));
        assert_eq!(stats.get("max"), Some(&30.0));
        assert_eq!(stats.get("count"), Some(&3.0));
    }

    #[test]
    fn test_filter_by_metadata() {
        let mut record1 = DataRecord::new(1, 1234567890);
        record1.add_metadata("type".to_string(), "a".to_string());

        let mut record2 = DataRecord::new(2, 1234567891);
        record2.add_metadata("type".to_string(), "b".to_string());

        let records = vec![record1, record2];
        let filtered = filter_by_metadata(&records, "type", "a");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1);
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

        for (line_number, line_result) in reader.lines().enumerate() {
            let line = line_result?;
            
            if line_number == 0 && self.has_header {
                continue;
            }

            if line.trim().is_empty() {
                continue;
            }

            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if fields.iter().all(|f| f.is_empty()) {
                continue;
            }

            records.push(fields);
        }

        Ok(records)
    }

    pub fn validate_record(&self, record: &[String]) -> bool {
        !record.is_empty() && record.iter().any(|field| !field.is_empty())
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
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_validate_record() {
        let processor = DataProcessor::new(',', false);
        let valid_record = vec!["data".to_string(), "value".to_string()];
        let empty_record = vec![];
        let blank_record = vec!["".to_string(), "".to_string()];

        assert!(processor.validate_record(&valid_record));
        assert!(!processor.validate_record(&empty_record));
        assert!(!processor.validate_record(&blank_record));
    }

    #[test]
    fn test_extract_column() {
        let records = vec![
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec!["d".to_string(), "e".to_string(), "f".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let column = processor.extract_column(&records, 1);
        
        assert_eq!(column, vec!["b".to_string(), "e".to_string()]);
    }
}use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u64, values: Vec<f64>) -> Self {
        Self {
            id,
            values,
            metadata: HashMap::new(),
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.values.is_empty() && self.id > 0
    }

    pub fn calculate_statistics(&self) -> Option<DataStats> {
        if self.values.is_empty() {
            return None;
        }

        let sum: f64 = self.values.iter().sum();
        let count = self.values.len() as f64;
        let mean = sum / count;

        let variance: f64 = self.values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;

        Some(DataStats {
            mean,
            variance,
            count: self.values.len(),
            min: *self.values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
            max: *self.values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
        })
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn transform_values<F>(&mut self, transformer: F)
    where
        F: Fn(f64) -> f64,
    {
        self.values = self.values.iter().map(|&x| transformer(x)).collect();
    }
}

#[derive(Debug, Clone)]
pub struct DataStats {
    pub mean: f64,
    pub variance: f64,
    pub count: usize,
    pub min: f64,
    pub max: f64,
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self { records: Vec::new() }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), String> {
        if !record.is_valid() {
            return Err("Invalid record data".to_string());
        }

        if self.records.iter().any(|r| r.id == record.id) {
            return Err("Duplicate record ID".to_string());
        }

        self.records.push(record);
        Ok(())
    }

    pub fn process_all(&mut self) -> ProcessingResult {
        let valid_count = self.records.iter().filter(|r| r.is_valid()).count();
        let invalid_count = self.records.len() - valid_count;

        let stats: Vec<DataStats> = self.records
            .iter()
            .filter_map(|r| r.calculate_statistics())
            .collect();

        let overall_mean = if !stats.is_empty() {
            stats.iter().map(|s| s.mean).sum::<f64>() / stats.len() as f64
        } else {
            0.0
        };

        ProcessingResult {
            total_records: self.records.len(),
            valid_records: valid_count,
            invalid_records: invalid_count,
            overall_mean,
            individual_stats: stats,
        }
    }

    pub fn filter_records<F>(&self, predicate: F) -> Vec<DataRecord>
    where
        F: Fn(&DataRecord) -> bool,
    {
        self.records
            .iter()
            .filter(|r| predicate(r))
            .cloned()
            .collect()
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[derive(Debug)]
pub struct ProcessingResult {
    pub total_records: usize,
    pub valid_records: usize,
    pub invalid_records: usize,
    pub overall_mean: f64,
    pub individual_stats: Vec<DataStats>,
}

impl ProcessingResult {
    pub fn is_successful(&self) -> bool {
        self.invalid_records == 0 && self.total_records > 0
    }

    pub fn summary(&self) -> String {
        format!(
            "Processed {} records ({} valid, {} invalid). Overall mean: {:.4}",
            self.total_records,
            self.valid_records,
            self.invalid_records,
            self.overall_mean
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, vec![1.0, 2.0, 3.0]);
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(0, vec![]);
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_statistics_calculation() {
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let stats = record.calculate_statistics().unwrap();

        assert_eq!(stats.mean, 3.0);
        assert_eq!(stats.variance, 2.0);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
        assert_eq!(stats.count, 5);
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();

        let record1 = DataRecord::new(1, vec![1.0, 2.0, 3.0]);
        let record2 = DataRecord::new(2, vec![4.0, 5.0, 6.0]);

        assert!(processor.add_record(record1).is_ok());
        assert!(processor.add_record(record2).is_ok());

        let result = processor.process_all();
        assert_eq!(result.total_records, 2);
        assert_eq!(result.valid_records, 2);
        assert_eq!(result.invalid_records, 0);
    }
}use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);
        
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        
        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    fn save_filtered_results<P: AsRef<Path>>(
        &self,
        category: &str,
        output_path: P,
    ) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        
        let file = File::create(output_path)?;
        let mut wtr = WriterBuilder::new().has_headers(true).from_writer(file);
        
        for record in filtered {
            wtr.serialize(record)?;
        }
        
        wtr.flush()?;
        Ok(())
    }

    fn add_record(&mut self, id: u32, name: String, value: f64, category: String) {
        self.records.push(Record {
            id,
            name,
            value,
            category,
        });
    }

    fn sort_by_value(&mut self) {
        self.records.sort_by(|a, b| a.value.partial_cmp(&b.value).unwrap());
    }
}

fn process_data_sample() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    processor.add_record(1, "ItemA".to_string(), 42.5, "Alpha".to_string());
    processor.add_record(2, "ItemB".to_string(), 33.2, "Beta".to_string());
    processor.add_record(3, "ItemC".to_string(), 56.8, "Alpha".to_string());
    processor.add_record(4, "ItemD".to_string(), 19.7, "Gamma".to_string());
    
    println!("Average value: {:.2}", processor.calculate_average());
    
    let alpha_items = processor.filter_by_category("Alpha");
    println!("Alpha category items: {}", alpha_items.len());
    
    processor.sort_by_value();
    println!("Records sorted by value");
    
    processor.save_filtered_results("Alpha", "alpha_results.csv")?;
    
    Ok(())
}use std::collections::HashMap;

pub struct DataProcessor {
    filters: Vec<Box<dyn Fn(&str) -> bool>>,
    transformations: HashMap<String, Box<dyn Fn(String) -> String>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            filters: Vec::new(),
            transformations: HashMap::new(),
        }
    }

    pub fn add_filter<F>(&mut self, filter: F)
    where
        F: Fn(&str) -> bool + 'static,
    {
        self.filters.push(Box::new(filter));
    }

    pub fn add_transformation<F>(&mut self, name: &str, transform: F)
    where
        F: Fn(String) -> String + 'static,
    {
        self.transformations
            .insert(name.to_string(), Box::new(transform));
    }

    pub fn process_data(&self, input: &str) -> Option<String> {
        if !self.filters.iter().all(|filter| filter(input)) {
            return None;
        }

        let mut result = input.to_string();
        for transform in self.transformations.values() {
            result = transform(result);
        }

        Some(result)
    }

    pub fn batch_process(&self, inputs: Vec<&str>) -> Vec<String> {
        inputs
            .iter()
            .filter_map(|&input| self.process_data(input))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();

        processor.add_filter(|s| s.len() > 3);
        processor.add_transformation("uppercase", |s| s.to_uppercase());
        processor.add_transformation("trim", |s| s.trim().to_string());

        let result = processor.process_data("  test data  ");
        assert_eq!(result, Some("TEST DATA".to_string()));

        let invalid_result = processor.process_data("abc");
        assert_eq!(invalid_result, None);

        let batch_results = processor.batch_process(vec!["  one  ", "two", "  three  "]);
        assert_eq!(batch_results, vec!["THREE"]);
    }
}
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Invalid input data: {0}")]
    InvalidInput(String),
    #[error("Processing timeout")]
    Timeout,
    #[error("Serialization failed")]
    SerializationFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u64, timestamp: i64) -> Self {
        Self {
            id,
            timestamp,
            values: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_value(&mut self, value: f64) -> &mut Self {
        self.values.push(value);
        self
    }

    pub fn add_metadata(&mut self, key: String, value: String) -> &mut Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn validate(&self) -> Result<(), DataError> {
        if self.id == 0 {
            return Err(DataError::InvalidInput("ID cannot be zero".to_string()));
        }
        
        if self.timestamp < 0 {
            return Err(DataError::InvalidInput("Timestamp cannot be negative".to_string()));
        }
        
        if self.values.is_empty() {
            return Err(DataError::InvalidInput("Values cannot be empty".to_string()));
        }

        Ok(())
    }

    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if self.values.is_empty() {
            return stats;
        }

        let sum: f64 = self.values.iter().sum();
        let count = self.values.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = self.values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;
        
        stats.insert("mean".to_string(), mean);
        stats.insert("sum".to_string(), sum);
        stats.insert("count".to_string(), count);
        stats.insert("variance".to_string(), variance);
        
        if let Some(&min) = self.values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()) {
            stats.insert("min".to_string(), min);
        }
        
        if let Some(&max) = self.values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()) {
            stats.insert("max".to_string(), max);
        }

        stats
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    processing_limit: Option<usize>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            processing_limit: None,
        }
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.processing_limit = Some(limit);
        self
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        record.validate()?;
        
        if let Some(limit) = self.processing_limit {
            if self.records.len() >= limit {
                return Err(DataError::InvalidInput(
                    format!("Exceeded processing limit of {}", limit)
                ));
            }
        }
        
        self.records.push(record);
        Ok(())
    }

    pub fn process_all(&self) -> HashMap<String, f64> {
        let mut aggregated_stats = HashMap::new();
        let mut total_count = 0;
        let mut total_sum = 0.0;

        for record in &self.records {
            let stats = record.calculate_statistics();
            
            if let Some(&sum) = stats.get("sum") {
                total_sum += sum;
            }
            
            if let Some(&count) = stats.get("count") {
                total_count += count as usize;
            }
        }

        if total_count > 0 {
            aggregated_stats.insert("total_mean".to_string(), total_sum / total_count as f64);
            aggregated_stats.insert("total_sum".to_string(), total_sum);
            aggregated_stats.insert("total_count".to_string(), total_count as f64);
        }

        aggregated_stats
    }

    pub fn filter_by_metadata(&self, key: &str, value: &str) -> Vec<DataRecord> {
        self.records.iter()
            .filter(|record| record.metadata.get(key) == Some(&value.to_string()))
            .cloned()
            .collect()
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
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
    fn test_record_validation() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value(42.0);
        
        assert!(record.validate().is_ok());
        
        let invalid_record = DataRecord::new(0, 1234567890);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value(10.0).add_value(20.0).add_value(30.0);
        
        let stats = record.calculate_statistics();
        
        assert_eq!(stats.get("mean"), Some(&20.0));
        assert_eq!(stats.get("sum"), Some(&60.0));
        assert_eq!(stats.get("count"), Some(&3.0));
        assert_eq!(stats.get("min"), Some(&10.0));
        assert_eq!(stats.get("max"), Some(&30.0));
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new().with_limit(2);
        
        let mut record1 = DataRecord::new(1, 1234567890);
        record1.add_value(10.0).add_metadata("type".to_string(), "test".to_string());
        
        let mut record2 = DataRecord::new(2, 1234567891);
        record2.add_value(20.0).add_metadata("type".to_string(), "test".to_string());
        
        assert!(processor.add_record(record1).is_ok());
        assert!(processor.add_record(record2).is_ok());
        
        let filtered = processor.filter_by_metadata("type", "test");
        assert_eq!(filtered.len(), 2);
        
        let stats = processor.process_all();
        assert_eq!(stats.get("total_sum"), Some(&30.0));
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
        let mut lines = reader.lines().enumerate();

        if self.has_header {
            let _ = lines.next();
        }

        for (line_number, line) in lines {
            let line = line?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if fields.iter().any(|f| f.is_empty()) {
                return Err(format!("Empty field detected at line {}", line_number + 1).into());
            }

            records.push(fields);
        }

        if records.is_empty() {
            return Err("No valid data records found".into());
        }

        Ok(records)
    }

    pub fn validate_records(&self, records: &[Vec<String>]) -> Result<(), Box<dyn Error>> {
        if records.is_empty() {
            return Err("Empty record set".into());
        }

        let expected_len = records[0].len();
        for (idx, record) in records.iter().enumerate() {
            if record.len() != expected_len {
                return Err(format!("Record {} has {} fields, expected {}", 
                    idx + 1, record.len(), expected_len).into());
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_valid_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,30,New York").unwrap();
        writeln!(temp_file, "Alice,25,London").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0], vec!["John", "30", "New York"]);
    }

    #[test]
    fn test_empty_field_detection() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "John,30,").unwrap();

        let processor = DataProcessor::new(',', false);
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_err());
    }
}use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

fn process_data(input_path: &str, output_path: &str, min_value: f64) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(input_file);

    let output_file = File::create(output_path)?;
    let mut writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(output_file);

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.value >= min_value {
            writer.serialize(&record)?;
        }
    }

    writer.flush()?;
    Ok(())
}

fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    let count = records.len() as f64;
    if count == 0.0 {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (sum, mean, std_dev)
}

fn filter_by_category(records: Vec<Record>, category: &str) -> Vec<Record> {
    records.into_iter()
        .filter(|r| r.category == category)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, category: "X".to_string() },
            Record { id: 2, name: "B".to_string(), value: 20.0, category: "Y".to_string() },
            Record { id: 3, name: "C".to_string(), value: 30.0, category: "X".to_string() },
        ];
        
        let (sum, mean, std_dev) = calculate_statistics(&records);
        
        assert_eq!(sum, 60.0);
        assert_eq!(mean, 20.0);
        assert!((std_dev - 8.164965).abs() < 0.0001);
    }

    #[test]
    fn test_category_filter() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, category: "X".to_string() },
            Record { id: 2, name: "B".to_string(), value: 20.0, category: "Y".to_string() },
            Record { id: 3, name: "C".to_string(), value: 30.0, category: "X".to_string() },
        ];
        
        let filtered = filter_by_category(records, "X");
        
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, 1);
        assert_eq!(filtered[1].id, 3);
    }
}