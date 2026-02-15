use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
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

            let category = parts[3].to_string();

            if !self.validate_record(&name, value) {
                continue;
            }

            self.records.push(DataRecord {
                id,
                name,
                value,
                category,
            });

            count += 1;
        }

        Ok(count)
    }

    fn validate_record(&self, name: &str, value: f64) -> bool {
        !name.is_empty() && value >= 0.0 && value <= 1000.0
    }

    pub fn get_records(&self) -> &[DataRecord] {
        &self.records
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn find_max_value(&self) -> Option<&DataRecord> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
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
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,100.5,Category1").unwrap();
        writeln!(temp_file, "2,ItemB,200.3,Category2").unwrap();
        writeln!(temp_file, "3,ItemC,150.7,Category1").unwrap();

        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);

        let records = processor.get_records();
        assert_eq!(records.len(), 3);

        let category1_items = processor.filter_by_category("Category1");
        assert_eq!(category1_items.len(), 2);

        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 150.5).abs() < 0.1);

        let max_record = processor.find_max_value();
        assert!(max_record.is_some());
        assert_eq!(max_record.unwrap().id, 2);
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
pub enum DataError {
    InvalidId,
    InvalidValue,
    EmptyName,
    UnknownCategory,
    DuplicateRecord,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidId => write!(f, "ID must be greater than zero"),
            DataError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            DataError::EmptyName => write!(f, "Name cannot be empty"),
            DataError::UnknownCategory => write!(f, "Category not recognized"),
            DataError::DuplicateRecord => write!(f, "Record with this ID already exists"),
        }
    }
}

impl Error for DataError {}

pub struct DataProcessor {
    records: HashMap<u32, DataRecord>,
    valid_categories: Vec<String>,
}

impl DataProcessor {
    pub fn new(categories: Vec<String>) -> Self {
        DataProcessor {
            records: HashMap::new(),
            valid_categories: categories,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), DataError> {
        if record.id == 0 {
            return Err(DataError::InvalidId);
        }

        if record.name.trim().is_empty() {
            return Err(DataError::EmptyName);
        }

        if record.value < 0.0 || record.value > 1000.0 {
            return Err(DataError::InvalidValue);
        }

        if !self.valid_categories.contains(&record.category) {
            return Err(DataError::UnknownCategory);
        }

        Ok(())
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), DataError> {
        self.validate_record(&record)?;

        if self.records.contains_key(&record.id) {
            return Err(DataError::DuplicateRecord);
        }

        self.records.insert(record.id, record);
        Ok(())
    }

    pub fn get_record(&self, id: u32) -> Option<&DataRecord> {
        self.records.get(&id)
    }

    pub fn calculate_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if self.records.is_empty() {
            return stats;
        }

        let count = self.records.len() as f64;
        let sum: f64 = self.records.values().map(|r| r.value).sum();
        let avg = sum / count;
        
        let min = self.records.values()
            .map(|r| r.value)
            .fold(f64::INFINITY, f64::min);
            
        let max = self.records.values()
            .map(|r| r.value)
            .fold(f64::NEG_INFINITY, f64::max);

        stats.insert("total_records".to_string(), count);
        stats.insert("sum".to_string(), sum);
        stats.insert("average".to_string(), avg);
        stats.insert("minimum".to_string(), min);
        stats.insert("maximum".to_string(), max);

        stats
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records.values()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn transform_values<F>(&mut self, transform_fn: F) 
    where
        F: Fn(f64) -> f64,
    {
        for record in self.records.values_mut() {
            record.value = transform_fn(record.value);
        }
    }

    pub fn export_as_csv(&self) -> String {
        let mut csv = String::from("id,name,value,category\n");
        
        for record in self.records.values() {
            csv.push_str(&format!("{},{},{},{}\n", 
                record.id, 
                record.name, 
                record.value, 
                record.category));
        }
        
        csv
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_validation() {
        let categories = vec!["A".to_string(), "B".to_string()];
        let processor = DataProcessor::new(categories);
        
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };
        
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_invalid_id_validation() {
        let categories = vec!["A".to_string()];
        let processor = DataProcessor::new(categories);
        
        let record = DataRecord {
            id: 0,
            name: "Test".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };
        
        assert!(matches!(processor.validate_record(&record), Err(DataError::InvalidId)));
    }

    #[test]
    fn test_record_addition_and_retrieval() {
        let mut processor = DataProcessor::new(vec!["Category1".to_string()]);
        
        let record = DataRecord {
            id: 42,
            name: "Sample".to_string(),
            value: 250.5,
            category: "Category1".to_string(),
        };
        
        assert!(processor.add_record(record.clone()).is_ok());
        assert_eq!(processor.get_record(42).unwrap().name, "Sample");
    }

    #[test]
    fn test_statistics_calculation() {
        let mut processor = DataProcessor::new(vec!["Test".to_string()]);
        
        let records = vec![
            DataRecord { id: 1, name: "A".to_string(), value: 10.0, category: "Test".to_string() },
            DataRecord { id: 2, name: "B".to_string(), value: 20.0, category: "Test".to_string() },
            DataRecord { id: 3, name: "C".to_string(), value: 30.0, category: "Test".to_string() },
        ];
        
        for record in records {
            processor.add_record(record).unwrap();
        }
        
        let stats = processor.calculate_statistics();
        assert_eq!(stats["total_records"], 3.0);
        assert_eq!(stats["sum"], 60.0);
        assert_eq!(stats["average"], 20.0);
        assert_eq!(stats["minimum"], 10.0);
        assert_eq!(stats["maximum"], 30.0);
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

    pub fn add_dataset(&mut self, name: &str, values: Vec<f64>) {
        self.data.insert(name.to_string(), values);
    }

    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }

    pub fn validate_all(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        for rule in &self.validation_rules {
            if let Some(data_values) = self.data.get(&rule.field_name) {
                if rule.required && data_values.is_empty() {
                    errors.push(format!("Field '{}' is required but empty", rule.field_name));
                }

                for &value in data_values {
                    if value < rule.min_value || value > rule.max_value {
                        errors.push(format!(
                            "Value {} in field '{}' is outside valid range [{}, {}]",
                            value, rule.field_name, rule.min_value, rule.max_value
                        ));
                    }
                }
            } else if rule.required {
                errors.push(format!("Required field '{}' not found", rule.field_name));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn normalize_data(&mut self, field_name: &str) -> Option<Vec<f64>> {
        if let Some(values) = self.data.get_mut(field_name) {
            if values.is_empty() {
                return None;
            }

            let min_val = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max_val = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            let range = max_val - min_val;

            if range == 0.0 {
                return Some(vec![0.0; values.len()]);
            }

            let normalized: Vec<f64> = values
                .iter()
                .map(|&v| (v - min_val) / range)
                .collect();

            self.data.insert(field_name.to_string(), normalized.clone());
            Some(normalized)
        } else {
            None
        }
    }

    pub fn calculate_statistics(&self, field_name: &str) -> Option<Statistics> {
        if let Some(values) = self.data.get(field_name) {
            if values.is_empty() {
                return None;
            }

            let sum: f64 = values.iter().sum();
            let count = values.len() as f64;
            let mean = sum / count;

            let variance: f64 = values
                .iter()
                .map(|&v| (v - mean).powi(2))
                .sum::<f64>() / count;

            let sorted_values = {
                let mut sorted = values.clone();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
                sorted
            };

            let median = if count as usize % 2 == 0 {
                let mid = count as usize / 2;
                (sorted_values[mid - 1] + sorted_values[mid]) / 2.0
            } else {
                sorted_values[count as usize / 2]
            };

            Some(Statistics {
                mean,
                median,
                variance,
                std_dev: variance.sqrt(),
                min: *sorted_values.first().unwrap(),
                max: *sorted_values.last().unwrap(),
                count: values.len(),
            })
        } else {
            None
        }
    }

    pub fn merge_datasets(&mut self, target_field: &str, source_fields: &[&str]) -> Option<Vec<f64>> {
        let mut merged_data = Vec::new();

        for &field in source_fields {
            if let Some(values) = self.data.get(field) {
                merged_data.extend(values);
            }
        }

        if !merged_data.is_empty() {
            self.data.insert(target_field.to_string(), merged_data.clone());
            Some(merged_data)
        } else {
            None
        }
    }
}

pub struct Statistics {
    pub mean: f64,
    pub median: f64,
    pub variance: f64,
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
    fn test_data_validation() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("temperature", vec![20.5, 22.0, 18.5, 25.0]);
        
        let rule = ValidationRule::new("temperature", 15.0, 30.0, true);
        processor.add_validation_rule(rule);

        assert!(processor.validate_all().is_ok());
    }

    #[test]
    fn test_normalization() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("scores", vec![10.0, 20.0, 30.0, 40.0]);
        
        let normalized = processor.normalize_data("scores").unwrap();
        assert_eq!(normalized, vec![0.0, 1.0/3.0, 2.0/3.0, 1.0]);
    }

    #[test]
    fn test_statistics_calculation() {
        let mut processor = DataProcessor::new();
        processor.add_dataset("values", vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        
        let stats = processor.calculate_statistics("values").unwrap();
        assert_eq!(stats.mean, 3.0);
        assert_eq!(stats.median, 3.0);
        assert_eq!(stats.variance, 2.0);
    }
}