
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub timestamp: String,
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

            let name = parts[1].to_string();
            
            let value = match parts[2].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let timestamp = parts[3].to_string();

            let record = DataRecord {
                id,
                name,
                value,
                timestamp,
            };

            self.records.push(record);
            count += 1;
        }

        Ok(count)
    }

    pub fn filter_by_value(&self, min_value: f64, max_value: f64) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.value >= min_value && record.value <= max_value)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn find_by_id(&self, target_id: u32) -> Option<&DataRecord> {
        self.records.iter().find(|record| record.id == target_id)
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
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
        assert_eq!(processor.record_count(), 0);

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,timestamp").unwrap();
        writeln!(temp_file, "1,test1,10.5,2023-01-01").unwrap();
        writeln!(temp_file, "2,test2,20.3,2023-01-02").unwrap();
        writeln!(temp_file, "3,test3,15.7,2023-01-03").unwrap();

        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(processor.record_count(), 3);

        let avg = processor.calculate_average();
        assert!(avg.is_some());
        let expected_avg = (10.5 + 20.3 + 15.7) / 3.0;
        assert!((avg.unwrap() - expected_avg).abs() < 0.0001);

        let filtered = processor.filter_by_value(15.0, 20.0);
        assert_eq!(filtered.len(), 2);

        let found = processor.find_by_id(2);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "test2");

        processor.clear();
        assert_eq!(processor.record_count(), 0);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    file_path: String,
    delimiter: char,
}

impl DataProcessor {
    pub fn new(file_path: &str) -> Self {
        DataProcessor {
            file_path: file_path.to_string(),
            delimiter: ',',
        }
    }

    pub fn with_delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = delimiter;
        self
    }

    pub fn process(&self) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut records = Vec::new();

        for line in reader.lines() {
            let line_content = line?;
            if line_content.trim().is_empty() {
                continue;
            }

            let fields: Vec<String> = line_content
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if !fields.is_empty() {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn filter_records<F>(&self, predicate: F) -> Result<Vec<Vec<String>>, Box<dyn Error>>
    where
        F: Fn(&[String]) -> bool,
    {
        let all_records = self.process()?;
        let filtered: Vec<Vec<String>> = all_records
            .into_iter()
            .filter(|record| predicate(record))
            .collect();

        Ok(filtered)
    }

    pub fn count_records(&self) -> Result<usize, Box<dyn Error>> {
        let records = self.process()?;
        Ok(records.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        writeln!(temp_file, "Charlie,35,Paris").unwrap();
        temp_file.flush().unwrap();
        temp_file
    }

    #[test]
    fn test_process_csv() {
        let temp_file = create_test_csv();
        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());

        let result = processor.process();
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0], vec!["name", "age", "city"]);
    }

    #[test]
    fn test_filter_records() {
        let temp_file = create_test_csv();
        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());

        let result = processor.filter_records(|record| {
            record.get(1).and_then(|age| age.parse::<i32>().ok()).map_or(false, |age| age > 30)
        });

        assert!(result.is_ok());
        let filtered = result.unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0], vec!["Charlie", "35", "Paris"]);
    }

    #[test]
    fn test_count_records() {
        let temp_file = create_test_csv();
        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());

        let count = processor.count_records();
        assert!(count.is_ok());
        assert_eq!(count.unwrap(), 3);
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

        let processed_data = self.apply_transformations(data);
        self.validate_data(&processed_data)?;
        
        self.cache.insert(dataset_name.to_string(), processed_data.clone());
        
        Ok(processed_data)
    }

    pub fn get_cached_data(&self, dataset_name: &str) -> Option<&Vec<f64>> {
        self.cache.get(dataset_name)
    }

    pub fn calculate_statistics(&self, dataset_name: &str) -> Option<DatasetStats> {
        self.cache.get(dataset_name).map(|data| {
            let sum: f64 = data.iter().sum();
            let mean = sum / data.len() as f64;
            let variance: f64 = data.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / data.len() as f64;
            
            DatasetStats {
                count: data.len(),
                mean,
                variance,
                min: *data.iter().fold(&f64::INFINITY, |a, b| a.min(b)),
                max: *data.iter().fold(&f64::NEG_INFINITY, |a, b| a.max(b)),
            }
        })
    }

    fn apply_transformations(&self, data: &[f64]) -> Vec<f64> {
        data.iter()
            .map(|&x| {
                if x < 0.0 {
                    x.abs()
                } else {
                    x
                }
            })
            .collect()
    }

    fn validate_data(&self, data: &[f64]) -> Result<(), String> {
        for rule in &self.validation_rules {
            if rule.required && data.is_empty() {
                return Err(format!("Field '{}' is required but empty", rule.field_name));
            }
            
            for &value in data {
                if value < rule.min_value || value > rule.max_value {
                    return Err(format!(
                        "Value {} for field '{}' is outside valid range [{}, {}]",
                        value, rule.field_name, rule.min_value, rule.max_value
                    ));
                }
            }
        }
        Ok(())
    }
}

pub struct DatasetStats {
    pub count: usize,
    pub mean: f64,
    pub variance: f64,
    pub min: f64,
    pub max: f64,
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

        let data = vec![25.5, 30.2, -10.0, 45.8];
        let result = processor.process_dataset("weather", &data);
        
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(processed.len(), 4);
        assert!(processed[2] == 10.0);
    }

    #[test]
    fn test_statistics_calculation() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        processor.process_dataset("test", &data).unwrap();
        let stats = processor.calculate_statistics("test").unwrap();
        
        assert_eq!(stats.count, 5);
        assert_eq!(stats.mean, 3.0);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
    }
}