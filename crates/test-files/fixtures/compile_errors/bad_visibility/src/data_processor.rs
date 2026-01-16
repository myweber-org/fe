
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

    pub fn add_validation_rule<F>(&mut self, name: &str, rule: F)
    where
        F: Fn(&DataRecord) -> Result<(), ProcessingError> + 'static,
    {
        self.validation_rules.insert(name.to_string(), Box::new(rule));
    }

    pub fn add_transformation<F>(&mut self, transform: F)
    where
        F: Fn(DataRecord) -> Result<DataRecord, ProcessingError> + 'static,
    {
        self.transformation_pipeline.push(Box::new(transform));
    }

    pub fn process_record(&self, mut record: DataRecord) -> Result<DataRecord, ProcessingError> {
        for (rule_name, rule) in &self.validation_rules {
            rule(&record).map_err(|e| {
                ProcessingError::ValidationError(format!("Rule '{}' failed: {}", rule_name, e))
            })?;
        }

        for (index, transform) in self.transformation_pipeline.iter().enumerate() {
            record = transform(record).map_err(|e| {
                ProcessingError::TransformationFailed(format!("Step {} failed: {}", index + 1, e))
            })?;
        }

        Ok(record)
    }

    pub fn process_batch(&self, records: Vec<DataRecord>) -> Result<Vec<DataRecord>, ProcessingError> {
        let mut results = Vec::with_capacity(records.len());
        
        for record in records {
            match self.process_record(record) {
                Ok(processed) => results.push(processed),
                Err(e) => return Err(ProcessingError::InvalidData(
                    format!("Batch processing failed: {}", e)
                )),
            }
        }
        
        Ok(results)
    }
}

fn validate_positive_value(record: &DataRecord) -> Result<(), ProcessingError> {
    if record.value < 0.0 {
        Err(ProcessingError::ValidationError(
            format!("Value {} is negative", record.value)
        ))
    } else {
        Ok(())
    }
}

fn normalize_name(record: DataRecord) -> Result<DataRecord, ProcessingError> {
    let normalized_name = record.name.trim().to_lowercase();
    if normalized_name.is_empty() {
        Err(ProcessingError::TransformationFailed(
            "Name cannot be empty after normalization".to_string()
        ))
    } else {
        Ok(DataRecord {
            name: normalized_name,
            ..record
        })
    }
}

fn add_default_tag(record: DataRecord) -> Result<DataRecord, ProcessingError> {
    let mut tags = record.tags;
    if !tags.contains(&"processed".to_string()) {
        tags.push("processed".to_string());
    }
    
    Ok(DataRecord { tags, ..record })
}

pub fn create_default_processor() -> DataProcessor {
    let mut processor = DataProcessor::new();
    
    processor.add_validation_rule("positive_value", validate_positive_value);
    processor.add_transformation(normalize_name);
    processor.add_transformation(add_default_tag);
    
    processor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positive_validation() {
        let record = DataRecord {
            id: 1,
            name: "test".to_string(),
            value: -5.0,
            tags: vec![],
        };
        
        assert!(validate_positive_value(&record).is_err());
    }

    #[test]
    fn test_normalization() {
        let record = DataRecord {
            id: 1,
            name: "  TEST  ".to_string(),
            value: 10.0,
            tags: vec![],
        };
        
        let result = normalize_name(record).unwrap();
        assert_eq!(result.name, "test");
    }

    #[test]
    fn test_full_processing() {
        let processor = create_default_processor();
        
        let record = DataRecord {
            id: 1,
            name: "  Sample Data  ".to_string(),
            value: 42.5,
            tags: vec!["original".to_string()],
        };
        
        let result = processor.process_record(record).unwrap();
        
        assert_eq!(result.name, "sample data");
        assert!(result.tags.contains(&"processed".to_string()));
        assert!(result.tags.contains(&"original".to_string()));
    }
}use std::error::Error;
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
            
            if !fields.is_empty() {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn validate_record(&self, record: &[String]) -> bool {
        !record.is_empty() && record.iter().all(|field| !field.is_empty())
    }

    pub fn extract_column(&self, data: &[Vec<String>], column_index: usize) -> Vec<String> {
        data.iter()
            .filter_map(|record| record.get(column_index))
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
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,25,New York").unwrap();
        writeln!(temp_file, "Alice,30,London").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["John", "25", "New York"]);
        assert!(processor.validate_record(&result[0]));
        
        let names = processor.extract_column(&result, 0);
        assert_eq!(names, vec!["John", "Alice"]);
    }
}