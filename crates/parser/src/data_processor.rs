use csv::Reader;
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

struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);
        
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

    fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }
}

fn process_data() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    processor.load_from_csv("data.csv")?;
    
    println!("Total records: {}", processor.records.len());
    println!("Average value: {:.2}", processor.calculate_average());
    
    if let Some(max_record) = processor.find_max_value() {
        println!("Max value record: {:?}", max_record);
    }
    
    let filtered = processor.filter_by_category("premium");
    println!("Premium records: {}", filtered.len());
    
    Ok(())
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    file_path: String,
}

impl DataProcessor {
    pub fn new(file_path: &str) -> Self {
        DataProcessor {
            file_path: file_path.to_string(),
        }
    }

    pub fn process_csv(&self, filter_column: usize, filter_value: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut results = Vec::new();
        let mut line_count = 0;

        for line in reader.lines() {
            let line = line?;
            line_count += 1;

            if line_count == 1 {
                continue;
            }

            let columns: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();

            if columns.len() > filter_column {
                if columns[filter_column] == filter_value {
                    results.push(columns);
                }
            }
        }

        Ok(results)
    }

    pub fn calculate_average(&self, column_index: usize) -> Result<f64, Box<dyn Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut sum = 0.0;
        let mut count = 0;
        let mut line_count = 0;

        for line in reader.lines() {
            let line = line?;
            line_count += 1;

            if line_count == 1 {
                continue;
            }

            let columns: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();

            if columns.len() > column_index {
                if let Ok(value) = columns[column_index].parse::<f64>() {
                    sum += value;
                    count += 1;
                }
            }
        }

        if count > 0 {
            Ok(sum / count as f64)
        } else {
            Ok(0.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value").unwrap();
        writeln!(temp_file, "1,apple,10.5").unwrap();
        writeln!(temp_file, "2,banana,15.3").unwrap();
        writeln!(temp_file, "3,apple,8.7").unwrap();

        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let result = processor.process_csv(1, "apple").unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0][0], "1");
        assert_eq!(result[1][0], "3");
    }

    #[test]
    fn test_calculate_average() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,price").unwrap();
        writeln!(temp_file, "1,10.0").unwrap();
        writeln!(temp_file, "2,20.0").unwrap();
        writeln!(temp_file, "3,30.0").unwrap();

        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let average = processor.calculate_average(1).unwrap();

        assert_eq!(average, 20.0);
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
            return Err("Empty dataset provided".to_string());
        }

        for rule in &self.validation_rules {
            if rule.required && data.iter().any(|&x| x.is_nan()) {
                return Err(format!("Missing values in field: {}", rule.field_name));
            }

            if data.iter().any(|&x| x < rule.min_value || x > rule.max_value) {
                return Err(format!("Values out of range for field: {}", rule.field_name));
            }
        }

        let processed_data: Vec<f64> = data
            .iter()
            .map(|&x| x * 2.0)
            .filter(|&x| x > 0.0)
            .collect();

        if processed_data.is_empty() {
            return Err("All values filtered out during processing".to_string());
        }

        self.cache.insert(dataset_name.to_string(), processed_data.clone());

        Ok(processed_data)
    }

    pub fn get_cached_data(&self, dataset_name: &str) -> Option<&Vec<f64>> {
        self.cache.get(dataset_name)
    }

    pub fn calculate_statistics(&self, dataset_name: &str) -> Option<Statistics> {
        self.cache.get(dataset_name).map(|data| {
            let sum: f64 = data.iter().sum();
            let count = data.len() as f64;
            let mean = sum / count;
            let variance: f64 = data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / count;
            let std_dev = variance.sqrt();

            Statistics {
                mean,
                std_dev,
                min: *data.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
                max: *data.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
                count: data.len(),
            }
        })
    }
}

pub struct Statistics {
    pub mean: f64,
    pub std_dev: f64,
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
        let rule = ValidationRule::new("temperature", -50.0, 100.0, true);
        processor.add_validation_rule(rule);

        let data = vec![10.0, 20.0, 30.0, 40.0];
        let result = processor.process_dataset("test_data", &data);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![20.0, 40.0, 60.0, 80.0]);
    }

    #[test]
    fn test_invalid_data() {
        let mut processor = DataProcessor::new();
        let rule = ValidationRule::new("pressure", 0.0, 100.0, true);
        processor.add_validation_rule(rule);

        let data = vec![50.0, 150.0, 30.0];
        let result = processor.process_dataset("invalid_data", &data);

        assert!(result.is_err());
    }
}
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
            Box::new(|s: &str| s.contains('@') && s.contains('.')),
        );
        
        self.validators.insert(
            "numeric".to_string(),
            Box::new(|s: &str| s.parse::<f64>().is_ok()),
        );
        
        self.validators.insert(
            "not_empty".to_string(),
            Box::new(|s: &str| !s.trim().is_empty()),
        );
    }
    
    fn register_default_transformers(&mut self) {
        self.transformers.insert(
            "uppercase".to_string(),
            Box::new(|s: String| s.to_uppercase()),
        );
        
        self.transformers.insert(
            "trim".to_string(),
            Box::new(|s: String| s.trim().to_string()),
        );
        
        self.transformers.insert(
            "reverse".to_string(),
            Box::new(|s: String| s.chars().rev().collect()),
        );
    }
    
    pub fn validate(&self, validator_name: &str, data: &str) -> bool {
        self.validators
            .get(validator_name)
            .map(|validator| validator(data))
            .unwrap_or(false)
    }
    
    pub fn transform(&self, transformer_name: &str, data: String) -> String {
        self.transformers
            .get(transformer_name)
            .map(|transformer| transformer(data))
            .unwrap_or(data)
    }
    
    pub fn process_pipeline(&self, data: &str, operations: Vec<(&str, &str)>) -> Result<String, String> {
        let mut result = data.to_string();
        
        for (op_type, op_name) in operations {
            match op_type {
                "validate" => {
                    if !self.validate(op_name, &result) {
                        return Err(format!("Validation '{}' failed for data: {}", op_name, result));
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
    
    pub fn register_validator<F>(&mut self, name: &str, validator: F)
    where
        F: Fn(&str) -> bool + 'static,
    {
        self.validators.insert(name.to_string(), Box::new(validator));
    }
    
    pub fn register_transformer<F>(&mut self, name: &str, transformer: F)
    where
        F: Fn(String) -> String + 'static,
    {
        self.transformers.insert(name.to_string(), Box::new(transformer));
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
        assert!(processor.validate("numeric", "123.45"));
        assert!(!processor.validate("numeric", "abc"));
    }
    
    #[test]
    fn test_transformation_pipeline() {
        let processor = DataProcessor::new();
        let result = processor.process_pipeline(
            "  hello  ",
            vec![
                ("validate", "not_empty"),
                ("transform", "trim"),
                ("transform", "uppercase"),
            ],
        );
        
        assert_eq!(result.unwrap(), "HELLO");
    }
    
    #[test]
    fn test_custom_validator() {
        let mut processor = DataProcessor::new();
        processor.register_validator("even_length", |s: &str| s.len() % 2 == 0);
        
        assert!(processor.validate("even_length", "ab"));
        assert!(!processor.validate("even_length", "abc"));
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

    pub fn filter_valid_records(&self, records: Vec<Vec<String>>) -> Vec<Vec<String>> {
        records
            .into_iter()
            .filter(|record| self.validate_record(record))
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
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        writeln!(temp_file, ",,Paris").unwrap();

        let processor = DataProcessor::new(',', true);
        let records = processor.process_file(temp_file.path()).unwrap();
        let valid_records = processor.filter_valid_records(records);

        assert_eq!(valid_records.len(), 2);
        assert_eq!(valid_records[0], vec!["Alice", "30", "New York"]);
        assert_eq!(valid_records[1], vec!["Bob", "25", "London"]);
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
    fn test_process_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        
        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "30", "New York"]);
        assert_eq!(result[1], vec!["Bob", "25", "London"]);
    }
    
    #[test]
    fn test_validate_record() {
        let processor = DataProcessor::new(',', false);
        let valid_record = vec!["data".to_string(), "value".to_string()];
        let invalid_record = vec!["".to_string(), "value".to_string()];
        
        assert!(processor.validate_record(&valid_record));
        assert!(!processor.validate_record(&invalid_record));
    }
    
    #[test]
    fn test_extract_column() {
        let data = vec![
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec!["d".to_string(), "e".to_string(), "f".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let column = processor.extract_column(&data, 1);
        
        assert_eq!(column, vec!["b".to_string(), "e".to_string()]);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    data: Vec<f64>,
    frequency_map: HashMap<String, u32>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: Vec::new(),
            frequency_map: HashMap::new(),
        }
    }

    pub fn load_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            
            for part in parts {
                if let Ok(value) = part.trim().parse::<f64>() {
                    self.data.push(value);
                } else {
                    self.frequency_map
                        .entry(part.trim().to_string())
                        .and_modify(|count| *count += 1)
                        .or_insert(1);
                }
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

    pub fn calculate_standard_deviation(&self) -> Option<f64> {
        if self.data.len() < 2 {
            return None;
        }
        
        let mean = self.calculate_mean()?;
        let variance: f64 = self.data
            .iter()
            .map(|value| {
                let diff = mean - *value;
                diff * diff
            })
            .sum::<f64>() / (self.data.len() - 1) as f64;
        
        Some(variance.sqrt())
    }

    pub fn get_top_categories(&self, limit: usize) -> Vec<(String, u32)> {
        let mut categories: Vec<_> = self.frequency_map.iter().collect();
        categories.sort_by(|a, b| b.1.cmp(a.1));
        
        categories
            .iter()
            .take(limit)
            .map(|(k, v)| (k.clone(), *v))
            .collect()
    }

    pub fn filter_data(&self, predicate: impl Fn(f64) -> bool) -> Vec<f64> {
        self.data
            .iter()
            .filter(|&&value| predicate(value))
            .copied()
            .collect()
    }

    pub fn data_summary(&self) -> String {
        let mean_str = self.calculate_mean()
            .map(|m| format!("{:.2}", m))
            .unwrap_or_else(|| "N/A".to_string());
        
        let std_dev_str = self.calculate_standard_deviation()
            .map(|s| format!("{:.2}", s))
            .unwrap_or_else(|| "N/A".to_string());
        
        format!(
            "Data points: {}, Mean: {}, Std Dev: {}, Categories: {}",
            self.data.len(),
            mean_str,
            std_dev_str,
            self.frequency_map.len()
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
        writeln!(temp_file, "10.5,20.3,15.7").unwrap();
        writeln!(temp_file, "category_a,25.1,category_b").unwrap();
        writeln!(temp_file, "18.9,category_a,12.4").unwrap();
        
        processor.load_csv(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.data.len(), 5);
        assert!(processor.calculate_mean().unwrap() > 0.0);
        assert!(processor.calculate_standard_deviation().unwrap() > 0.0);
        
        let top_categories = processor.get_top_categories(2);
        assert_eq!(top_categories.len(), 2);
        assert_eq!(top_categories[0].0, "category_a");
        assert_eq!(top_categories[0].1, 2);
    }
}use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        Ok(())
    }
}

pub fn process_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let path = Path::new(input_path);
    if !path.exists() {
        return Err("Input file does not exist".into());
    }

    let mut reader = Reader::from_path(input_path)?;
    let mut writer = Writer::from_path(output_path)?;

    let mut valid_count = 0;
    let mut invalid_count = 0;

    for result in reader.deserialize() {
        let record: Record = result?;
        
        match record.validate() {
            Ok(_) => {
                writer.serialize(&record)?;
                valid_count += 1;
            }
            Err(err) => {
                eprintln!("Invalid record {}: {}", record.id, err);
                invalid_count += 1;
            }
        }
    }

    writer.flush()?;
    
    println!("Processing complete:");
    println!("  Valid records: {}", valid_count);
    println!("  Invalid records: {}", invalid_count);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_record_validation() {
        let valid_record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 10.5,
            active: true,
        };
        assert!(valid_record.validate().is_ok());

        let invalid_record = Record {
            id: 2,
            name: "".to_string(),
            value: -5.0,
            active: false,
        };
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_csv_processing() {
        let test_input = "test_input.csv";
        let test_output = "test_output.csv";
        
        let csv_data = "id,name,value,active\n1,Alice,100.5,true\n2,Bob,-50.0,false\n3,,75.0,true\n";
        fs::write(test_input, csv_data).unwrap();

        let result = process_csv(test_input, test_output);
        assert!(result.is_ok());

        fs::remove_file(test_input).unwrap();
        fs::remove_file(test_output).unwrap();
    }
}