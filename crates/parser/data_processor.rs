use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub valid: bool,
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

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if index == 0 {
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

            let category = parts[2].to_string();
            let valid = value >= 0.0 && value <= 1000.0;

            self.records.push(DataRecord {
                id,
                value,
                category,
                valid,
            });

            count += 1;
        }

        Ok(count)
    }

    pub fn filter_valid(&self) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|r| r.valid)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records: Vec<&DataRecord> = self.records.iter().filter(|r| r.valid).collect();
        
        if valid_records.is_empty() {
            return None;
        }

        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<DataRecord>> {
        let mut groups = std::collections::HashMap::new();
        
        for record in &self.records {
            if record.valid {
                groups
                    .entry(record.category.clone())
                    .or_insert_with(Vec::new)
                    .push(record.clone());
            }
        }
        
        groups
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn count_valid(&self) -> usize {
        self.records.iter().filter(|r| r.valid).count()
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
        writeln!(temp_file, "1,100.5,CategoryA").unwrap();
        writeln!(temp_file, "2,200.3,CategoryB").unwrap();
        writeln!(temp_file, "3,1500.0,CategoryA").unwrap();
        
        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(processor.count_records(), 3);
        assert_eq!(processor.count_valid(), 2);
        
        let average = processor.calculate_average();
        assert!(average.is_some());
        assert!((average.unwrap() - 150.4).abs() < 0.0001);
        
        let groups = processor.group_by_category();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups.get("CategoryA").unwrap().len(), 1);
        assert_eq!(groups.get("CategoryB").unwrap().len(), 1);
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

    pub fn process_csv<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines();

        if self.has_header {
            lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !record.is_empty() {
                records.push(record);
            }
        }

        Ok(records)
    }

    pub fn validate_records(&self, records: &[Vec<String>]) -> Result<(), String> {
        if records.is_empty() {
            return Err("No records found".to_string());
        }

        let expected_len = records[0].len();
        for (i, record) in records.iter().enumerate() {
            if record.len() != expected_len {
                return Err(format!("Record {} has {} fields, expected {}", i + 1, record.len(), expected_len));
            }
        }

        Ok(())
    }

    pub fn calculate_column_averages(&self, records: &[Vec<String>]) -> Result<Vec<f64>, String> {
        if records.is_empty() {
            return Err("No records to process".to_string());
        }

        let column_count = records[0].len();
        let mut sums = vec![0.0; column_count];
        let mut counts = vec![0; column_count];

        for record in records {
            for (i, field) in record.iter().enumerate() {
                if let Ok(value) = field.parse::<f64>() {
                    sums[i] += value;
                    counts[i] += 1;
                }
            }
        }

        let averages: Vec<f64> = sums
            .iter()
            .zip(counts.iter())
            .map(|(&sum, &count)| if count > 0 { sum / count as f64 } else { 0.0 })
            .collect();

        Ok(averages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000.5").unwrap();
        writeln!(temp_file, "Bob,25,45000.0").unwrap();
        writeln!(temp_file, "Charlie,35,55000.75").unwrap();

        let processor = DataProcessor::new(',', true);
        let records = processor.process_csv(temp_file.path()).unwrap();
        
        assert_eq!(records.len(), 3);
        assert_eq!(records[0], vec!["Alice", "30", "50000.5"]);
    }

    #[test]
    fn test_validation() {
        let valid_records = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string(), "d".to_string()],
        ];
        
        let invalid_records = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string()],
        ];

        let processor = DataProcessor::new(',', false);
        
        assert!(processor.validate_records(&valid_records).is_ok());
        assert!(processor.validate_records(&invalid_records).is_err());
    }

    #[test]
    fn test_average_calculation() {
        let records = vec![
            vec!["10".to_string(), "20".to_string()],
            vec!["20".to_string(), "30".to_string()],
            vec!["30".to_string(), "40".to_string()],
        ];

        let processor = DataProcessor::new(',', false);
        let averages = processor.calculate_column_averages(&records).unwrap();
        
        assert_eq!(averages, vec![20.0, 30.0]);
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ProcessingError {
    InvalidInput(String),
    TransformationFailed(String),
    ValidationError(String),
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessingError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            ProcessingError::TransformationFailed(msg) => write!(f, "Transformation failed: {}", msg),
            ProcessingError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ProcessingError {}

pub struct DataProcessor {
    threshold: f64,
    max_items: usize,
}

impl DataProcessor {
    pub fn new(threshold: f64, max_items: usize) -> Result<Self, ProcessingError> {
        if threshold < 0.0 || threshold > 1.0 {
            return Err(ProcessingError::InvalidInput(
                "Threshold must be between 0.0 and 1.0".to_string(),
            ));
        }
        if max_items == 0 {
            return Err(ProcessingError::InvalidInput(
                "Max items must be greater than zero".to_string(),
            ));
        }
        Ok(DataProcessor {
            threshold,
            max_items,
        })
    }

    pub fn process_data(&self, input: &[f64]) -> Result<Vec<f64>, ProcessingError> {
        if input.len() > self.max_items {
            return Err(ProcessingError::ValidationError(format!(
                "Input length {} exceeds maximum allowed {}",
                input.len(),
                self.max_items
            )));
        }

        let filtered: Vec<f64> = input
            .iter()
            .filter(|&&value| value >= self.threshold)
            .cloned()
            .collect();

        if filtered.is_empty() {
            return Err(ProcessingError::TransformationFailed(
                "No values meet the threshold criteria".to_string(),
            ));
        }

        let normalized = self.normalize_values(&filtered)?;
        Ok(normalized)
    }

    fn normalize_values(&self, values: &[f64]) -> Result<Vec<f64>, ProcessingError> {
        let max_value = values
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        if max_value <= 0.0 {
            return Err(ProcessingError::TransformationFailed(
                "Cannot normalize non-positive values".to_string(),
            ));
        }

        let normalized: Vec<f64> = values
            .iter()
            .map(|&value| value / max_value)
            .collect();

        Ok(normalized)
    }

    pub fn calculate_statistics(&self, data: &[f64]) -> Result<(f64, f64), ProcessingError> {
        if data.is_empty() {
            return Err(ProcessingError::InvalidInput(
                "Cannot calculate statistics for empty dataset".to_string(),
            ));
        }

        let sum: f64 = data.iter().sum();
        let mean = sum / data.len() as f64;

        let variance: f64 = data
            .iter()
            .map(|&value| {
                let diff = value - mean;
                diff * diff
            })
            .sum::<f64>()
            / data.len() as f64;

        let std_dev = variance.sqrt();
        Ok((mean, std_dev))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_creation() {
        let processor = DataProcessor::new(0.5, 100);
        assert!(processor.is_ok());

        let invalid_processor = DataProcessor::new(1.5, 100);
        assert!(invalid_processor.is_err());
    }

    #[test]
    fn test_data_processing() {
        let processor = DataProcessor::new(0.3, 10).unwrap();
        let input = vec![0.1, 0.4, 0.5, 0.2, 0.8];
        let result = processor.process_data(&input);
        assert!(result.is_ok());

        let processed = result.unwrap();
        assert_eq!(processed.len(), 3);
        assert!(processed.iter().all(|&x| x >= 0.0 && x <= 1.0));
    }

    #[test]
    fn test_statistics_calculation() {
        let processor = DataProcessor::new(0.0, 100).unwrap();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = processor.calculate_statistics(&data);
        assert!(stats.is_ok());

        let (mean, std_dev) = stats.unwrap();
        assert_eq!(mean, 3.0);
        assert!(std_dev > 1.41 && std_dev < 1.42);
    }
}