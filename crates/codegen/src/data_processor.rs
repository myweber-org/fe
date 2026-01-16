
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

    pub fn process_data(&mut self, dataset: &[HashMap<String, f64>]) -> Result<Vec<ProcessedRecord>, String> {
        let mut results = Vec::new();

        for (index, record) in dataset.iter().enumerate() {
            match self.validate_record(record) {
                Ok(_) => {
                    let processed = self.transform_record(record);
                    self.cache.insert(format!("record_{}", index), processed.values.clone());
                    results.push(processed);
                }
                Err(e) => return Err(format!("Validation failed at record {}: {}", index, e)),
            }
        }

        Ok(results)
    }

    fn validate_record(&self, record: &HashMap<String, f64>) -> Result<(), String> {
        for rule in &self.validation_rules {
            if let Some(&value) = record.get(&rule.field_name) {
                if value < rule.min_value || value > rule.max_value {
                    return Err(format!(
                        "Field '{}' value {} out of range [{}, {}]",
                        rule.field_name, value, rule.min_value, rule.max_value
                    ));
                }
            } else if rule.required {
                return Err(format!("Required field '{}' missing", rule.field_name));
            }
        }
        Ok(())
    }

    fn transform_record(&self, record: &HashMap<String, f64>) -> ProcessedRecord {
        let mut values = Vec::new();
        let mut stats = HashMap::new();

        for (key, &value) in record {
            let transformed = (value * 100.0).round() / 100.0;
            values.push(transformed);
            stats.insert(key.clone(), transformed);
        }

        ProcessedRecord {
            values,
            statistics: stats,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn get_cached_data(&self, key: &str) -> Option<&Vec<f64>> {
        self.cache.get(key)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

pub struct ProcessedRecord {
    pub values: Vec<f64>,
    pub statistics: HashMap<String, f64>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
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
        processor.add_validation_rule(ValidationRule::new("temperature", -50.0, 150.0, true));
        
        let mut record = HashMap::new();
        record.insert("temperature".to_string(), 25.5);
        record.insert("humidity".to_string(), 65.2);
        
        let dataset = vec![record];
        let result = processor.process_data(&dataset);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[test]
    fn test_validation_failure() {
        let mut processor = DataProcessor::new();
        processor.add_validation_rule(ValidationRule::new("pressure", 900.0, 1100.0, true));
        
        let mut record = HashMap::new();
        record.insert("pressure".to_string(), 1200.0);
        
        let dataset = vec![record];
        let result = processor.process_data(&dataset);
        
        assert!(result.is_err());
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

    pub fn process_file<P: AsRef<Path>>(
        &self,
        file_path: P,
        filter_predicate: Option<Box<dyn Fn(&[String]) -> bool>>,
    ) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        if self.has_header {
            lines.next();
        }

        let mut results = Vec::new();

        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if let Some(ref predicate) = filter_predicate {
                if predicate(&fields) {
                    results.push(fields);
                }
            } else {
                results.push(fields);
            }
        }

        Ok(results)
    }

    pub fn calculate_column_average(
        &self,
        data: &[Vec<String>],
        column_index: usize,
    ) -> Result<f64, Box<dyn Error>> {
        let mut sum = 0.0;
        let mut count = 0;

        for row in data {
            if column_index < row.len() {
                if let Ok(value) = row[column_index].parse::<f64>() {
                    sum += value;
                    count += 1;
                }
            }
        }

        if count > 0 {
            Ok(sum / count as f64)
        } else {
            Err("No valid numeric data found in specified column".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000.0").unwrap();
        writeln!(temp_file, "Bob,25,45000.0").unwrap();
        writeln!(temp_file, "Charlie,35,55000.0").unwrap();

        let processor = DataProcessor::new(',', true);
        let results = processor
            .process_file(temp_file.path(), None)
            .unwrap();

        assert_eq!(results.len(), 3);
        assert_eq!(results[0][0], "Alice");
        assert_eq!(results[1][1], "25");

        let avg_age = processor.calculate_column_average(&results, 1).unwrap();
        assert!((avg_age - 30.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_filtered_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value").unwrap();
        writeln!(temp_file, "1,100").unwrap();
        writeln!(temp_file, "2,200").unwrap();
        writeln!(temp_file, "3,50").unwrap();

        let processor = DataProcessor::new(',', true);
        let filter = Box::new(|fields: &[String]| {
            if fields.len() > 1 {
                fields[1].parse::<i32>().unwrap_or(0) > 100
            } else {
                false
            }
        });

        let results = processor
            .process_file(temp_file.path(), Some(filter))
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0][1], "200");
    }
}