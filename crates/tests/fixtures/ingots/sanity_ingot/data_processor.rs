
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

    pub fn validate_records(&self, records: &[Vec<String>]) -> Result<(), String> {
        if records.is_empty() {
            return Err("No valid records found".to_string());
        }

        let expected_len = records[0].len();
        for (i, record) in records.iter().enumerate() {
            if record.len() != expected_len {
                return Err(format!("Record {} has {} fields, expected {}", 
                    i + 1, record.len(), expected_len));
            }
        }

        Ok(())
    }

    pub fn extract_column(&self, records: &[Vec<String>], column_index: usize) -> Result<Vec<String>, String> {
        if column_index >= records[0].len() {
            return Err(format!("Column index {} out of bounds", column_index));
        }

        let column_data: Vec<String> = records
            .iter()
            .map(|record| record[column_index].clone())
            .collect();

        Ok(column_data)
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
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0], vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_validate_records() {
        let records = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string(), "d".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        assert!(processor.validate_records(&records).is_ok());
    }

    #[test]
    fn test_extract_column() {
        let records = vec![
            vec!["Alice".to_string(), "30".to_string()],
            vec!["Bob".to_string(), "25".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let column = processor.extract_column(&records, 1).unwrap();
        assert_eq!(column, vec!["30", "25"]);
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

        let validated_data = self.validate_data(data)?;
        let transformed_data = self.transform_data(&validated_data);
        
        self.cache.insert(dataset_name.to_string(), transformed_data.clone());
        
        Ok(transformed_data)
    }

    fn validate_data(&self, data: &[f64]) -> Result<Vec<f64>, String> {
        for rule in &self.validation_rules {
            for &value in data {
                if value < rule.min_value || value > rule.max_value {
                    return Err(format!(
                        "Value {} out of range for field {}: [{}, {}]",
                        value, rule.field_name, rule.min_value, rule.max_value
                    ));
                }
            }
        }
        Ok(data.to_vec())
    }

    fn transform_data(&self, data: &[f64]) -> Vec<f64> {
        let mean = self.calculate_mean(data);
        let std_dev = self.calculate_std_dev(data, mean);
        
        data.iter()
            .map(|&x| (x - mean) / std_dev)
            .collect()
    }

    fn calculate_mean(&self, data: &[f64]) -> f64 {
        data.iter().sum::<f64>() / data.len() as f64
    }

    fn calculate_std_dev(&self, data: &[f64], mean: f64) -> f64 {
        let variance = data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64;
        
        variance.sqrt()
    }

    pub fn get_cached_result(&self, dataset_name: &str) -> Option<&Vec<f64>> {
        self.cache.get(dataset_name)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
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
        let rule = ValidationRule::new("temperature", -50.0, 150.0, true);
        processor.add_validation_rule(rule);

        let data = vec![20.5, 25.0, 30.2, 18.7];
        let result = processor.process_dataset("test_data", &data);
        
        assert!(result.is_ok());
        assert_eq!(processor.get_cached_result("test_data").unwrap().len(), 4);
    }

    #[test]
    fn test_validation_failure() {
        let mut processor = DataProcessor::new();
        let rule = ValidationRule::new("pressure", 0.0, 100.0, true);
        processor.add_validation_rule(rule);

        let data = vec![50.0, 75.0, 120.0];
        let result = processor.process_dataset("invalid_data", &data);
        
        assert!(result.is_err());
    }
}