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

        for rule in &self.validation_rules {
            if rule.required && data.iter().any(|&x| x.is_nan()) {
                return Err(format!("Field {} contains invalid values", rule.field_name));
            }
        }

        let processed: Vec<f64> = data.iter()
            .map(|&x| {
                if x.is_nan() {
                    0.0
                } else {
                    x * 2.0
                }
            })
            .collect();

        self.cache.insert(dataset_name.to_string(), processed.clone());
        Ok(processed)
    }

    pub fn get_cached_data(&self, dataset_name: &str) -> Option<&Vec<f64>> {
        self.cache.get(dataset_name)
    }

    pub fn calculate_statistics(&self, dataset_name: &str) -> Option<DatasetStats> {
        self.cache.get(dataset_name).map(|data| {
            let sum: f64 = data.iter().sum();
            let count = data.len() as f64;
            let mean = sum / count;
            
            let variance: f64 = data.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / count;
            
            DatasetStats {
                mean,
                variance,
                min: *data.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
                max: *data.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
                count: data.len(),
            }
        })
    }
}

pub struct DatasetStats {
    pub mean: f64,
    pub variance: f64,
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
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        let result = processor.process_dataset("test_data", &data);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed, vec![2.0, 4.0, 6.0, 8.0, 10.0]);
    }

    #[test]
    fn test_empty_dataset() {
        let mut processor = DataProcessor::new();
        let result = processor.process_dataset("empty", &[]);
        assert!(result.is_err());
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub timestamp: u64,
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
            let timestamp = match parts[3].parse::<u64>() {
                Ok(val) => val,
                Err(_) => continue,
            };
            
            self.records.push(DataRecord {
                id,
                value,
                category,
                timestamp,
            });
            
            count += 1;
        }
        
        Ok(count)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn get_records_count(&self) -> usize {
        self.records.len()
    }

    pub fn get_record_by_id(&self, id: u32) -> Option<&DataRecord> {
        self.records.iter().find(|r| r.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor_creation() {
        let processor = DataProcessor::new();
        assert_eq!(processor.get_records_count(), 0);
    }

    #[test]
    fn test_load_csv() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category,timestamp").unwrap();
        writeln!(temp_file, "1,42.5,alpha,1625097600").unwrap();
        writeln!(temp_file, "2,37.8,beta,1625184000").unwrap();
        writeln!(temp_file, "3,55.2,alpha,1625270400").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(processor.get_records_count(), 3);
    }

    #[test]
    fn test_filter_by_category() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category,timestamp").unwrap();
        writeln!(temp_file, "1,42.5,alpha,1625097600").unwrap();
        writeln!(temp_file, "2,37.8,beta,1625184000").unwrap();
        writeln!(temp_file, "3,55.2,alpha,1625270400").unwrap();
        
        processor.load_from_csv(temp_file.path().to_str().unwrap()).unwrap();
        
        let alpha_records = processor.filter_by_category("alpha");
        assert_eq!(alpha_records.len(), 2);
        
        let beta_records = processor.filter_by_category("beta");
        assert_eq!(beta_records.len(), 1);
    }

    #[test]
    fn test_calculate_average() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category,timestamp").unwrap();
        writeln!(temp_file, "1,10.0,alpha,1625097600").unwrap();
        writeln!(temp_file, "2,20.0,beta,1625184000").unwrap();
        writeln!(temp_file, "3,30.0,alpha,1625270400").unwrap();
        
        processor.load_from_csv(temp_file.path().to_str().unwrap()).unwrap();
        
        let average = processor.calculate_average();
        assert!(average.is_some());
        assert_eq!(average.unwrap(), 20.0);
    }

    #[test]
    fn test_get_record_by_id() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category,timestamp").unwrap();
        writeln!(temp_file, "1,42.5,alpha,1625097600").unwrap();
        writeln!(temp_file, "2,37.8,beta,1625184000").unwrap();
        
        processor.load_from_csv(temp_file.path().to_str().unwrap()).unwrap();
        
        let record = processor.get_record_by_id(2);
        assert!(record.is_some());
        assert_eq!(record.unwrap().value, 37.8);
        assert_eq!(record.unwrap().category, "beta");
    }
}