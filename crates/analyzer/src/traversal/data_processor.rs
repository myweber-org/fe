
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
            
            if !self.validate_record(id, value, &category) {
                continue;
            }
            
            self.records.push(DataRecord {
                id,
                value,
                category,
            });
            
            count += 1;
        }
        
        Ok(count)
    }
    
    fn validate_record(&self, id: u32, value: f64, category: &str) -> bool {
        if id == 0 {
            return false;
        }
        
        if value < 0.0 || value > 10000.0 {
            return false;
        }
        
        let valid_categories = ["A", "B", "C", "D"];
        valid_categories.contains(&category)
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
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,100.5,A").unwrap();
        writeln!(temp_file, "2,200.0,B").unwrap();
        writeln!(temp_file, "3,300.75,C").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(processor.record_count(), 3);
        
        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 200.41666666666666).abs() < 0.0001);
        
        let filtered = processor.filter_by_category("A");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1);
        
        let stats = processor.get_statistics();
        assert!((stats.0 - 100.5).abs() < 0.0001);
        assert!((stats.1 - 300.75).abs() < 0.0001);
    }
    
    #[test]
    fn test_validation() {
        let processor = DataProcessor::new();
        
        assert!(processor.validate_record(1, 50.0, "A"));
        assert!(!processor.validate_record(0, 50.0, "A"));
        assert!(!processor.validate_record(1, -10.0, "A"));
        assert!(!processor.validate_record(1, 50000.0, "A"));
        assert!(!processor.validate_record(1, 50.0, "X"));
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    data: HashMap<String, Vec<f64>>,
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
            data: HashMap::new(),
            validation_rules: Vec::new(),
        }
    }

    pub fn add_dataset(&mut self, name: String, values: Vec<f64>) -> Result<(), String> {
        if name.is_empty() {
            return Err("Dataset name cannot be empty".to_string());
        }
        
        if values.is_empty() {
            return Err("Dataset values cannot be empty".to_string());
        }

        self.data.insert(name, values);
        Ok(())
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }

    pub fn validate_all(&self) -> Vec<ValidationResult> {
        let mut results = Vec::new();
        
        for rule in &self.validation_rules {
            if let Some(values) = self.data.get(&rule.field_name) {
                let result = self.validate_dataset(values, rule);
                results.push(result);
            } else if rule.required {
                results.push(ValidationResult {
                    field_name: rule.field_name.clone(),
                    valid: false,
                    message: "Required field not found".to_string(),
                });
            }
        }
        
        results
    }

    fn validate_dataset(&self, values: &[f64], rule: &ValidationRule) -> ValidationResult {
        let mut valid = true;
        let mut message = String::new();
        
        for (index, &value) in values.iter().enumerate() {
            if value < rule.min_value || value > rule.max_value {
                valid = false;
                message = format!("Value {} at index {} is out of range [{}, {}]", 
                    value, index, rule.min_value, rule.max_value);
                break;
            }
        }
        
        if valid {
            message = "All values are within valid range".to_string();
        }
        
        ValidationResult {
            field_name: rule.field_name.clone(),
            valid,
            message,
        }
    }

    pub fn calculate_statistics(&self, dataset_name: &str) -> Option<Statistics> {
        self.data.get(dataset_name).map(|values| {
            let count = values.len();
            let sum: f64 = values.iter().sum();
            let mean = sum / count as f64;
            
            let variance: f64 = values.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count as f64;
            
            let std_dev = variance.sqrt();
            
            let mut sorted_values = values.clone();
            sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
            
            let median = if count % 2 == 0 {
                (sorted_values[count / 2 - 1] + sorted_values[count / 2]) / 2.0
            } else {
                sorted_values[count / 2]
            };
            
            Statistics {
                count,
                mean,
                median,
                std_dev,
                min: *sorted_values.first().unwrap_or(&0.0),
                max: *sorted_values.last().unwrap_or(&0.0),
            }
        })
    }
}

pub struct ValidationResult {
    field_name: String,
    valid: bool,
    message: String,
}

pub struct Statistics {
    count: usize,
    mean: f64,
    median: f64,
    std_dev: f64,
    min: f64,
    max: f64,
}

impl ValidationRule {
    pub fn new(field_name: String, min_value: f64, max_value: f64, required: bool) -> Self {
        ValidationRule {
            field_name,
            min_value,
            max_value,
            required,
        }
    }
}