
use std::collections::HashMap;

pub struct DataProcessor {
    data: HashMap<String, Vec<f64>>,
    validation_rules: HashMap<String, ValidationRule>,
}

pub struct ValidationRule {
    min_value: Option<f64>,
    max_value: Option<f64>,
    required: bool,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: HashMap::new(),
            validation_rules: HashMap::new(),
        }
    }

    pub fn add_dataset(&mut self, key: &str, values: Vec<f64>) -> Result<(), String> {
        if let Some(rule) = self.validation_rules.get(key) {
            if rule.required && values.is_empty() {
                return Err(format!("Dataset '{}' cannot be empty", key));
            }

            for &value in &values {
                if let Some(min) = rule.min_value {
                    if value < min {
                        return Err(format!("Value {} below minimum {} for dataset '{}'", value, min, key));
                    }
                }
                if let Some(max) = rule.max_value {
                    if value > max {
                        return Err(format!("Value {} above maximum {} for dataset '{}'", value, max, key));
                    }
                }
            }
        }

        self.data.insert(key.to_string(), values);
        Ok(())
    }

    pub fn set_validation_rule(&mut self, key: &str, rule: ValidationRule) {
        self.validation_rules.insert(key.to_string(), rule);
    }

    pub fn calculate_statistics(&self, key: &str) -> Option<Statistics> {
        self.data.get(key).map(|values| {
            let count = values.len();
            if count == 0 {
                return Statistics::empty();
            }

            let sum: f64 = values.iter().sum();
            let mean = sum / count as f64;
            
            let variance: f64 = values.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count as f64;
            
            let std_dev = variance.sqrt();

            let sorted_values = {
                let mut sorted = values.clone();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
                sorted
            };

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
                min: *sorted_values.first().unwrap(),
                max: *sorted_values.last().unwrap(),
            }
        })
    }

    pub fn normalize_data(&mut self, key: &str) -> Result<(), String> {
        if let Some(values) = self.data.get_mut(key) {
            if values.is_empty() {
                return Err("Cannot normalize empty dataset".to_string());
            }

            let stats = self.calculate_statistics(key).unwrap();
            
            if stats.std_dev == 0.0 {
                return Err("Cannot normalize dataset with zero standard deviation".to_string());
            }

            for value in values.iter_mut() {
                *value = (*value - stats.mean) / stats.std_dev;
            }
            Ok(())
        } else {
            Err(format!("Dataset '{}' not found", key))
        }
    }

    pub fn get_data(&self, key: &str) -> Option<&Vec<f64>> {
        self.data.get(key)
    }
}

pub struct Statistics {
    pub count: usize,
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
}

impl Statistics {
    fn empty() -> Self {
        Statistics {
            count: 0,
            mean: 0.0,
            median: 0.0,
            std_dev: 0.0,
            min: 0.0,
            max: 0.0,
        }
    }
}

impl ValidationRule {
    pub fn new(min_value: Option<f64>, max_value: Option<f64>, required: bool) -> Self {
        ValidationRule {
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
        
        let rule = ValidationRule::new(Some(0.0), Some(100.0), true);
        processor.set_validation_rule("scores", rule);
        
        let data = vec![85.5, 92.0, 78.5, 95.0];
        assert!(processor.add_dataset("scores", data).is_ok());
        
        let stats = processor.calculate_statistics("scores").unwrap();
        assert_eq!(stats.count, 4);
        assert!(stats.mean > 87.0 && stats.mean < 88.0);
        
        assert!(processor.normalize_data("scores").is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let mut processor = DataProcessor::new();
        
        let rule = ValidationRule::new(Some(0.0), Some(100.0), true);
        processor.set_validation_rule("scores", rule);
        
        let invalid_data = vec![85.5, 105.0, 78.5];
        assert!(processor.add_dataset("scores", invalid_data).is_err());
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str) -> Result<Self, String> {
        if value < 0.0 {
            return Err("Value cannot be negative".to_string());
        }
        if category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(Self {
            id,
            value,
            category: category.to_string(),
        })
    }

    pub fn calculate_adjusted_value(&self, multiplier: f64) -> f64 {
        self.value * multiplier
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut loaded_count = 0;

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

            match DataRecord::new(id, value, parts[2]) {
                Ok(record) => {
                    self.records.push(record);
                    loaded_count += 1;
                }
                Err(_) => continue,
            }
        }

        Ok(loaded_count)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|record| record.value).sum()
    }

    pub fn get_average_value(&self) -> Option<f64> {
        if self.records.is_empty() {
            None
        } else {
            Some(self.calculate_total_value() / self.records.len() as f64)
        }
    }

    pub fn find_max_value_record(&self) -> Option<&DataRecord> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 42.5, "test").unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "test");
    }

    #[test]
    fn test_invalid_data_record() {
        assert!(DataRecord::new(1, -5.0, "test").is_err());
        assert!(DataRecord::new(1, 5.0, "").is_err());
    }

    #[test]
    fn test_calculate_adjusted_value() {
        let record = DataRecord::new(1, 10.0, "test").unwrap();
        assert_eq!(record.calculate_adjusted_value(2.0), 20.0);
    }

    #[test]
    fn test_data_processor_operations() {
        let mut processor = DataProcessor::new();
        
        let temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,category_a").unwrap();
        writeln!(temp_file, "2,20.3,category_b").unwrap();
        writeln!(temp_file, "3,15.7,category_a").unwrap();

        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);

        let filtered = processor.filter_by_category("category_a");
        assert_eq!(filtered.len(), 2);

        let total = processor.calculate_total_value();
        assert!((total - 46.5).abs() < 0.001);

        let average = processor.get_average_value().unwrap();
        assert!((average - 15.5).abs() < 0.001);

        let max_record = processor.find_max_value_record().unwrap();
        assert_eq!(max_record.id, 2);
    }
}