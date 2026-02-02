
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
pub enum ValidationError {
    InvalidId,
    EmptyName,
    NegativeValue,
    DuplicateTag,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidId => write!(f, "ID must be greater than 0"),
            ValidationError::EmptyName => write!(f, "Name cannot be empty"),
            ValidationError::NegativeValue => write!(f, "Value cannot be negative"),
            ValidationError::DuplicateTag => write!(f, "Duplicate tags found"),
        }
    }
}

impl Error for ValidationError {}

pub fn validate_record(record: &DataRecord) -> Result<(), ValidationError> {
    if record.id == 0 {
        return Err(ValidationError::InvalidId);
    }
    
    if record.name.trim().is_empty() {
        return Err(ValidationError::EmptyName);
    }
    
    if record.value < 0.0 {
        return Err(ValidationError::NegativeValue);
    }
    
    let mut seen_tags = HashMap::new();
    for tag in &record.tags {
        if seen_tags.contains_key(tag) {
            return Err(ValidationError::DuplicateTag);
        }
        seen_tags.insert(tag.clone(), true);
    }
    
    Ok(())
}

pub fn transform_records(records: Vec<DataRecord>) -> Vec<DataRecord> {
    records
        .into_iter()
        .filter(|record| record.value > 10.0)
        .map(|mut record| {
            record.name = record.name.to_uppercase();
            record.tags.sort();
            record.tags.dedup();
            record
        })
        .collect()
}

pub fn calculate_statistics(records: &[DataRecord]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = sum / count;
    
    let variance: f64 = records
        .iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_record_valid() {
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 15.5,
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        };
        
        assert!(validate_record(&record).is_ok());
    }
    
    #[test]
    fn test_validate_record_invalid_id() {
        let record = DataRecord {
            id: 0,
            name: "Test".to_string(),
            value: 15.5,
            tags: vec![],
        };
        
        assert!(matches!(validate_record(&record), Err(ValidationError::InvalidId)));
    }
    
    #[test]
    fn test_transform_records() {
        let records = vec![
            DataRecord {
                id: 1,
                name: "alpha".to_string(),
                value: 5.0,
                tags: vec!["z".to_string(), "a".to_string(), "z".to_string()],
            },
            DataRecord {
                id: 2,
                name: "beta".to_string(),
                value: 20.0,
                tags: vec!["b".to_string(), "a".to_string()],
            },
        ];
        
        let transformed = transform_records(records);
        assert_eq!(transformed.len(), 1);
        assert_eq!(transformed[0].name, "BETA");
        assert_eq!(transformed[0].tags, vec!["a", "b"]);
    }
    
    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            DataRecord {
                id: 1,
                name: "test1".to_string(),
                value: 10.0,
                tags: vec![],
            },
            DataRecord {
                id: 2,
                name: "test2".to_string(),
                value: 20.0,
                tags: vec![],
            },
            DataRecord {
                id: 3,
                name: "test3".to_string(),
                value: 30.0,
                tags: vec![],
            },
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
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

    pub fn load_numeric_data(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if let Ok(value) = line.trim().parse::<f64>() {
                self.data.push(value);
            }
        }
        Ok(())
    }

    pub fn load_categorical_data(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            let category = line.trim().to_string();
            *self.frequency_map.entry(category).or_insert(0) += 1;
        }
        Ok(())
    }

    pub fn calculate_statistics(&self) -> (f64, f64, f64, f64) {
        if self.data.is_empty() {
            return (0.0, 0.0, 0.0, 0.0);
        }

        let sum: f64 = self.data.iter().sum();
        let count = self.data.len() as f64;
        let mean = sum / count;

        let variance: f64 = self.data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        let mut sorted_data = self.data.clone();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let median = if count as usize % 2 == 0 {
            let mid = count as usize / 2;
            (sorted_data[mid - 1] + sorted_data[mid]) / 2.0
        } else {
            sorted_data[count as usize / 2]
        };

        (mean, median, variance, std_dev)
    }

    pub fn get_frequency_distribution(&self) -> Vec<(&String, &usize)> {
        let mut entries: Vec<_> = self.frequency_map.iter().collect();
        entries.sort_by(|a, b| b.1.cmp(a.1));
        entries
    }

    pub fn filter_data(&self, threshold: f64) -> Vec<f64> {
        self.data.iter()
            .filter(|&&x| x > threshold)
            .cloned()
            .collect()
    }

    pub fn normalize_data(&mut self) {
        if self.data.is_empty() {
            return;
        }

        let (mean, _, _, std_dev) = self.calculate_statistics();
        
        if std_dev > 0.0 {
            for value in &mut self.data {
                *value = (*value - mean) / std_dev;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_numeric_data_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "10.5\n20.3\n15.7\n25.1\n18.9").unwrap();

        let mut processor = DataProcessor::new();
        processor.load_numeric_data(temp_file.path().to_str().unwrap()).unwrap();

        let (mean, median, variance, std_dev) = processor.calculate_statistics();
        
        assert!((mean - 18.1).abs() < 0.1);
        assert!((median - 18.9).abs() < 0.1);
        assert!(variance > 0.0);
        assert!(std_dev > 0.0);
    }

    #[test]
    fn test_categorical_data_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "apple\nbanana\napple\norange\nbanana\nbanana").unwrap();

        let mut processor = DataProcessor::new();
        processor.load_categorical_data(temp_file.path().to_str().unwrap()).unwrap();

        let distribution = processor.get_frequency_distribution();
        assert_eq!(distribution[0].0, "banana");
        assert_eq!(*distribution[0].1, 3);
    }

    #[test]
    fn test_data_filtering() {
        let mut processor = DataProcessor::new();
        processor.data = vec![5.0, 15.0, 25.0, 35.0, 45.0];
        
        let filtered = processor.filter_data(20.0);
        assert_eq!(filtered, vec![25.0, 35.0, 45.0]);
    }
}