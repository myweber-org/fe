
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
    InvalidValue,
    UnknownCategory,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than 0"),
            ValidationError::EmptyName => write!(f, "Name cannot be empty"),
            ValidationError::InvalidValue => write!(f, "Value must be between 0.0 and 1000.0"),
            ValidationError::UnknownCategory => write!(f, "Category is not recognized"),
        }
    }
}

impl Error for ValidationError {}

pub struct DataProcessor {
    valid_categories: HashMap<String, bool>,
}

impl DataProcessor {
    pub fn new() -> Self {
        let mut categories = HashMap::new();
        categories.insert("A".to_string(), true);
        categories.insert("B".to_string(), true);
        categories.insert("C".to_string(), true);
        
        DataProcessor {
            valid_categories: categories,
        }
    }

    pub fn validate_record(&self, record: &DataRecord) -> Result<(), ValidationError> {
        if record.id == 0 {
            return Err(ValidationError::InvalidId);
        }
        
        if record.name.trim().is_empty() {
            return Err(ValidationError::EmptyName);
        }
        
        if record.value < 0.0 || record.value > 1000.0 {
            return Err(ValidationError::InvalidValue);
        }
        
        if !self.valid_categories.contains_key(&record.category) {
            return Err(ValidationError::UnknownCategory);
        }
        
        Ok(())
    }

    pub fn process_records(&self, records: Vec<DataRecord>) -> Vec<Result<DataRecord, ValidationError>> {
        records
            .into_iter()
            .map(|record| {
                self.validate_record(&record)
                    .map(|_| {
                        let processed_record = DataRecord {
                            value: (record.value * 100.0).round() / 100.0,
                            ..record
                        };
                        processed_record
                    })
            })
            .collect()
    }

    pub fn calculate_statistics(&self, records: &[DataRecord]) -> (f64, f64, f64) {
        if records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = records.iter().map(|r| r.value).sum();
        let count = records.len() as f64;
        let average = sum / count;
        
        let variance: f64 = records
            .iter()
            .map(|r| (r.value - average).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        (average, variance, std_dev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let processor = DataProcessor::new();
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 50.0,
            category: "A".to_string(),
        };
        
        assert!(processor.validate_record(&record).is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let processor = DataProcessor::new();
        let record = DataRecord {
            id: 0,
            name: "".to_string(),
            value: -10.0,
            category: "X".to_string(),
        };
        
        assert!(processor.validate_record(&record).is_err());
    }

    #[test]
    fn test_process_records() {
        let processor = DataProcessor::new();
        let records = vec![
            DataRecord {
                id: 1,
                name: "Record1".to_string(),
                value: 123.456,
                category: "A".to_string(),
            },
            DataRecord {
                id: 2,
                name: "Record2".to_string(),
                value: 78.912,
                category: "B".to_string(),
            },
        ];
        
        let results = processor.process_records(records);
        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
        
        if let Ok(record) = &results[0] {
            assert_eq!(record.value, 123.46);
        }
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new();
        let records = vec![
            DataRecord {
                id: 1,
                name: "R1".to_string(),
                value: 10.0,
                category: "A".to_string(),
            },
            DataRecord {
                id: 2,
                name: "R2".to_string(),
                value: 20.0,
                category: "B".to_string(),
            },
            DataRecord {
                id: 3,
                name: "R3".to_string(),
                value: 30.0,
                category: "C".to_string(),
            },
        ];
        
        let (avg, var, std) = processor.calculate_statistics(&records);
        assert_eq!(avg, 20.0);
        assert_eq!(var, 66.66666666666667);
        assert_eq!(std, 8.16496580927726);
    }
}
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0
    }
}

pub fn process_csv(input_path: &str, output_path: &str) -> Result<usize, Box<dyn Error>> {
    let input_file = File::open(Path::new(input_path))?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(input_file);

    let output_file = File::create(Path::new(output_path))?;
    let mut writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(output_file);

    let mut valid_count = 0;

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.is_valid() {
            writer.serialize(&record)?;
            valid_count += 1;
        }
    }

    writer.flush()?;
    Ok(valid_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_record() {
        let record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 42.5,
            active: true,
        };
        assert!(record.is_valid());
    }

    #[test]
    fn test_invalid_record() {
        let record = Record {
            id: 2,
            name: "".to_string(),
            value: -10.0,
            active: false,
        };
        assert!(!record.is_valid());
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    data: Vec<f64>,
    frequency_map: HashMap<String, usize>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: Vec::new(),
            frequency_map: HashMap::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        for line in reader.lines().skip(1) {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            
            if parts.len() >= 2 {
                if let Ok(value) = parts[1].parse::<f64>() {
                    self.data.push(value);
                }
                
                let category = parts[0].to_string();
                *self.frequency_map.entry(category).or_insert(0) += 1;
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

    pub fn get_frequency_distribution(&self) -> &HashMap<String, usize> {
        &self.frequency_map
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<f64> {
        self.data.iter()
            .filter(|&&x| x > threshold)
            .cloned()
            .collect()
    }

    pub fn data_summary(&self) -> String {
        let mean = self.calculate_mean().unwrap_or(0.0);
        let count = self.data.len();
        let unique_categories = self.frequency_map.len();
        
        format!(
            "Data Summary: {} records, {} unique categories, mean: {:.2}",
            count, unique_categories, mean
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
        writeln!(temp_file, "category,value").unwrap();
        writeln!(temp_file, "A,10.5").unwrap();
        writeln!(temp_file, "B,20.3").unwrap();
        writeln!(temp_file, "A,15.7").unwrap();
        writeln!(temp_file, "C,8.9").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        assert_eq!(processor.data.len(), 4);
        assert_eq!(processor.calculate_mean(), Some(13.85));
        
        let filtered = processor.filter_by_threshold(12.0);
        assert_eq!(filtered.len(), 2);
        
        let freq = processor.get_frequency_distribution();
        assert_eq!(freq.get("A"), Some(&2));
    }
}