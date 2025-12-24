use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

struct DataProcessor {
    input_path: String,
    output_path: String,
}

impl DataProcessor {
    fn new(input_path: &str, output_path: &str) -> Self {
        DataProcessor {
            input_path: input_path.to_string(),
            output_path: output_path.to_string(),
        }
    }

    fn process_data(&self, min_value: f64) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(reader);

        let output_file = File::create(&self.output_path)?;
        let writer = BufWriter::new(output_file);
        let mut csv_writer = WriterBuilder::new()
            .has_headers(true)
            .from_writer(writer);

        let mut processed_count = 0;

        for result in csv_reader.deserialize() {
            let record: Record = result?;
            
            if record.value >= min_value && record.active {
                csv_writer.serialize(&record)?;
                processed_count += 1;
            }
        }

        csv_writer.flush()?;
        Ok(processed_count)
    }

    fn calculate_statistics(&self) -> Result<(f64, f64, usize), Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(reader);

        let mut total = 0.0;
        let mut count = 0;
        let mut max_value = f64::MIN;

        for result in csv_reader.deserialize() {
            let record: Record = result?;
            
            if record.active {
                total += record.value;
                count += 1;
                if record.value > max_value {
                    max_value = record.value;
                }
            }
        }

        let average = if count > 0 { total / count as f64 } else { 0.0 };
        Ok((average, max_value, count))
    }
}

fn validate_file_path(path: &str) -> bool {
    Path::new(path).exists()
}

pub fn run_processing(input_path: &str, output_path: &str, threshold: f64) -> Result<(), Box<dyn Error>> {
    if !validate_file_path(input_path) {
        return Err("Input file does not exist".into());
    }

    let processor = DataProcessor::new(input_path, output_path);
    
    match processor.process_data(threshold) {
        Ok(count) => {
            println!("Processed {} records", count);
            
            match processor.calculate_statistics() {
                Ok((avg, max, total)) => {
                    println!("Statistics - Average: {:.2}, Max: {:.2}, Total active records: {}", avg, max, total);
                }
                Err(e) => eprintln!("Failed to calculate statistics: {}", e),
            }
        }
        Err(e) => eprintln!("Processing failed: {}", e),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_data_processing() {
        let input_data = "id,name,category,value,active\n\
                          1,ItemA,Category1,25.5,true\n\
                          2,ItemB,Category2,15.0,false\n\
                          3,ItemC,Category1,30.0,true\n";

        let mut input_file = NamedTempFile::new().unwrap();
        write!(input_file, "{}", input_data).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        let processor = DataProcessor::new(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap()
        );
        
        let result = processor.process_data(20.0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
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
pub enum ValidationError {
    InvalidId,
    EmptyName,
    NegativeValue,
    InvalidCategory,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than 0"),
            ValidationError::EmptyName => write!(f, "Name cannot be empty"),
            ValidationError::NegativeValue => write!(f, "Value cannot be negative"),
            ValidationError::InvalidCategory => write!(f, "Category must be one of: A, B, C"),
        }
    }
}

impl Error for ValidationError {}

pub fn validate_record(record: &DataRecord) -> Result<(), ValidationError> {
    if record.id == 0 {
        return Err(ValidationError::InvalidId);
    }
    
    if record.name.trim().is_empty() {
        return Err(ValidationError::EmptyName);
    }
    
    if record.value < 0.0 {
        return Err(ValidationError::NegativeValue);
    }
    
    let valid_categories = ["A", "B", "C"];
    if !valid_categories.contains(&record.category.as_str()) {
        return Err(ValidationError::InvalidCategory);
    }
    
    Ok(())
}

pub fn transform_records(records: Vec<DataRecord>) -> HashMap<String, Vec<DataRecord>> {
    let mut grouped = HashMap::new();
    
    for record in records {
        grouped
            .entry(record.category.clone())
            .or_insert_with(Vec::new)
            .push(record);
    }
    
    grouped
}

pub fn calculate_statistics(records: &[DataRecord]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_record_valid() {
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 10.5,
            category: "A".to_string(),
        };
        
        assert!(validate_record(&record).is_ok());
    }
    
    #[test]
    fn test_validate_record_invalid_id() {
        let record = DataRecord {
            id: 0,
            name: "Test".to_string(),
            value: 10.5,
            category: "A".to_string(),
        };
        
        assert!(matches!(validate_record(&record), Err(ValidationError::InvalidId)));
    }
    
    #[test]
    fn test_transform_records() {
        let records = vec![
            DataRecord {
                id: 1,
                name: "Item1".to_string(),
                value: 10.0,
                category: "A".to_string(),
            },
            DataRecord {
                id: 2,
                name: "Item2".to_string(),
                value: 20.0,
                category: "B".to_string(),
            },
            DataRecord {
                id: 3,
                name: "Item3".to_string(),
                value: 30.0,
                category: "A".to_string(),
            },
        ];
        
        let grouped = transform_records(records);
        assert_eq!(grouped.get("A").unwrap().len(), 2);
        assert_eq!(grouped.get("B").unwrap().len(), 1);
        assert!(grouped.get("C").is_none());
    }
    
    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            DataRecord {
                id: 1,
                name: "Item1".to_string(),
                value: 10.0,
                category: "A".to_string(),
            },
            DataRecord {
                id: 2,
                name: "Item2".to_string(),
                value: 20.0,
                category: "A".to_string(),
            },
            DataRecord {
                id: 3,
                name: "Item3".to_string(),
                value: 30.0,
                category: "A".to_string(),
            },
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
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
    validation_rules: HashMap<String, Box<dyn Fn(&DataRecord) -> Result<(), ProcessingError>>>,
    transformation_pipeline: Vec<Box<dyn Fn(DataRecord) -> Result<DataRecord, ProcessingError>>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            validation_rules: HashMap::new(),
            transformation_pipeline: Vec::new(),
        }
    }

    pub fn add_validation_rule(
        &mut self,
        name: &str,
        rule: Box<dyn Fn(&DataRecord) -> Result<(), ProcessingError>>,
    ) {
        self.validation_rules.insert(name.to_string(), rule);
    }

    pub fn add_transformation(
        &mut self,
        transform: Box<dyn Fn(DataRecord) -> Result<DataRecord, ProcessingError>>,
    ) {
        self.transformation_pipeline.push(transform);
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ProcessingError> {
        for (rule_name, rule) in &self.validation_rules {
            rule(record).map_err(|e| {
                ProcessingError::ValidationError(format!("Rule '{}' failed: {}", rule_name, e))
            })?;
        }
        Ok(())
    }

    pub fn process_record(&self, mut record: DataRecord) -> Result<DataRecord, ProcessingError> {
        self.validate_record(&record)?;

        for (i, transform) in self.transformation_pipeline.iter().enumerate() {
            record = transform(record).map_err(|e| {
                ProcessingError::TransformationFailed(format!("Step {} failed: {}", i + 1, e))
            })?;
        }

        Ok(record)
    }

    pub fn process_batch(
        &self,
        records: Vec<DataRecord>,
    ) -> Result<Vec<DataRecord>, Vec<(usize, ProcessingError)>> {
        let mut processed = Vec::with_capacity(records.len());
        let mut errors = Vec::new();

        for (index, record) in records.into_iter().enumerate() {
            match self.process_record(record) {
                Ok(processed_record) => processed.push(processed_record),
                Err(e) => errors.push((index, e)),
            }
        }

        if errors.is_empty() {
            Ok(processed)
        } else {
            Err(errors)
        }
    }
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();

    processor.add_validation_rule(
        "value_positive",
        Box::new(|record| {
            if record.value >= 0.0 {
                Ok(())
            } else {
                Err(ProcessingError::InvalidData(
                    "Value must be non-negative".to_string(),
                ))
            }
        }),
    );

    processor.add_validation_rule(
        "name_not_empty",
        Box::new(|record| {
            if !record.name.trim().is_empty() {
                Ok(())
            } else {
                Err(ProcessingError::InvalidData(
                    "Name cannot be empty".to_string(),
                ))
            }
        }),
    );

    processor.add_transformation(Box::new(|mut record| {
        record.name = record.name.trim().to_uppercase();
        record.tags = record
            .tags
            .into_iter()
            .map(|tag| tag.trim().to_lowercase())
            .collect();
        Ok(record)
    }));

    processor.add_transformation(Box::new(|mut record| {
        if record.value > 1000.0 {
            record.value = record.value.log10();
        }
        Ok(record)
    }));

    processor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let processor = create_default_processor();
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 42.0,
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        };

        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let processor = create_default_processor();
        let record = DataRecord {
            id: 1,
            name: "".to_string(),
            value: -5.0,
            tags: vec![],
        };

        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_transformation_pipeline() {
        let processor = create_default_processor();
        let record = DataRecord {
            id: 1,
            name: "  test data  ".to_string(),
            value: 2000.0,
            tags: vec!["  TAG1  ".to_string(), "TAG2".to_string()],
        };

        let result = processor.process_record(record).unwrap();
        assert_eq!(result.name, "TEST DATA");
        assert!((result.value - 3.30103).abs() < 0.00001);
        assert_eq!(result.tags, vec!["tag1", "tag2"]);
    }

    #[test]
    fn test_batch_processing() {
        let processor = create_default_processor();
        let records = vec![
            DataRecord {
                id: 1,
                name: "record1".to_string(),
                value: 100.0,
                tags: vec![],
            },
            DataRecord {
                id: 2,
                name: "".to_string(),
                value: 200.0,
                tags: vec![],
            },
            DataRecord {
                id: 3,
                name: "record3".to_string(),
                value: -50.0,
                tags: vec![],
            },
        ];

        let result = processor.process_batch(records);
        match result {
            Ok(_) => panic!("Expected errors"),
            Err(errors) => assert_eq!(errors.len(), 2),
        }
    }
}