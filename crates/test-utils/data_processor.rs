
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub timestamp: i64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u64, timestamp: i64) -> Self {
        Self {
            id,
            timestamp,
            values: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_value(&mut self, value: f64) -> &mut Self {
        self.values.push(value);
        self
    }

    pub fn add_metadata(&mut self, key: &str, value: &str) -> &mut Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0 {
            return Err("Invalid record ID".to_string());
        }
        if self.timestamp < 0 {
            return Err("Timestamp cannot be negative".to_string());
        }
        if self.values.is_empty() {
            return Err("Record must contain at least one value".to_string());
        }
        Ok(())
    }

    pub fn calculate_statistics(&self) -> Option<DataStatistics> {
        if self.values.is_empty() {
            return None;
        }

        let count = self.values.len();
        let sum: f64 = self.values.iter().sum();
        let mean = sum / count as f64;
        let min = self.values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        let variance: f64 = self.values
            .iter()
            .map(|value| {
                let diff = mean - value;
                diff * diff
            })
            .sum::<f64>() / count as f64;

        Some(DataStatistics {
            count,
            sum,
            mean,
            min,
            max,
            variance,
            std_dev: variance.sqrt(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct DataStatistics {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub min: f64,
    pub max: f64,
    pub variance: f64,
    pub std_dev: f64,
}

pub fn process_records(records: &[DataRecord]) -> Vec<Result<DataStatistics, String>> {
    records
        .iter()
        .map(|record| {
            record.validate()?;
            record.calculate_statistics()
                .ok_or_else(|| "Failed to calculate statistics".to_string())
        })
        .collect()
}

pub fn filter_valid_records(records: &[DataRecord]) -> Vec<&DataRecord> {
    records
        .iter()
        .filter(|record| record.validate().is_ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_creation() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value(42.5).add_value(37.2);
        record.add_metadata("source", "sensor_a");

        assert_eq!(record.id, 1);
        assert_eq!(record.timestamp, 1234567890);
        assert_eq!(record.values.len(), 2);
        assert_eq!(record.metadata.get("source"), Some(&"sensor_a".to_string()));
    }

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 1234567890).add_value(42.5);
        assert!(valid_record.validate().is_ok());

        let invalid_id = DataRecord::new(0, 1234567890).add_value(42.5);
        assert!(invalid_id.validate().is_err());

        let invalid_timestamp = DataRecord::new(1, -1).add_value(42.5);
        assert!(invalid_timestamp.validate().is_err());

        let no_values = DataRecord::new(1, 1234567890);
        assert!(no_values.validate().is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let mut record = DataRecord::new(1, 1234567890);
        record.add_value(10.0).add_value(20.0).add_value(30.0);

        let stats = record.calculate_statistics().unwrap();
        assert_eq!(stats.count, 3);
        assert_eq!(stats.sum, 60.0);
        assert_eq!(stats.mean, 20.0);
        assert_eq!(stats.min, 10.0);
        assert_eq!(stats.max, 30.0);
    }

    #[test]
    fn test_process_records() {
        let mut record1 = DataRecord::new(1, 1234567890);
        record1.add_value(10.0).add_value(20.0);

        let mut record2 = DataRecord::new(2, 1234567891);
        record2.add_value(30.0).add_value(40.0);

        let records = vec![record1, record2];
        let results = process_records(&records);

        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub struct DataSet {
    values: Vec<f64>,
}

impl DataSet {
    pub fn new() -> Self {
        DataSet { values: Vec::new() }
    }

    pub fn from_csv<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        let mut values = Vec::new();
        for line in contents.lines() {
            if let Ok(num) = line.trim().parse::<f64>() {
                values.push(num);
            }
        }

        Ok(DataSet { values })
    }

    pub fn add_value(&mut self, value: f64) {
        self.values.push(value);
    }

    pub fn mean(&self) -> Option<f64> {
        if self.values.is_empty() {
            return None;
        }
        let sum: f64 = self.values.iter().sum();
        Some(sum / self.values.len() as f64)
    }

    pub fn variance(&self) -> Option<f64> {
        if self.values.len() < 2 {
            return None;
        }
        let mean = self.mean().unwrap();
        let sum_sq_diff: f64 = self.values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum();
        Some(sum_sq_diff / (self.values.len() - 1) as f64)
    }

    pub fn min(&self) -> Option<f64> {
        self.values.iter().copied().reduce(f64::min)
    }

    pub fn max(&self) -> Option<f64> {
        self.values.iter().copied().reduce(f64::max)
    }

    pub fn count(&self) -> usize {
        self.values.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_empty_dataset() {
        let ds = DataSet::new();
        assert_eq!(ds.mean(), None);
        assert_eq!(ds.variance(), None);
        assert_eq!(ds.count(), 0);
    }

    #[test]
    fn test_basic_statistics() {
        let mut ds = DataSet::new();
        ds.add_value(1.0);
        ds.add_value(2.0);
        ds.add_value(3.0);
        ds.add_value(4.0);
        ds.add_value(5.0);

        assert_eq!(ds.mean(), Some(3.0));
        assert_eq!(ds.variance(), Some(2.5));
        assert_eq!(ds.min(), Some(1.0));
        assert_eq!(ds.max(), Some(5.0));
        assert_eq!(ds.count(), 5);
    }

    #[test]
    fn test_csv_parsing() -> Result<(), Box<dyn Error>> {
        let mut tmp_file = NamedTempFile::new()?;
        writeln!(tmp_file, "1.5\n2.5\n3.5\n4.5\n5.5")?;
        
        let ds = DataSet::from_csv(tmp_file.path())?;
        assert_eq!(ds.mean(), Some(3.5));
        assert_eq!(ds.count(), 5);
        Ok(())
    }

    #[test]
    fn test_invalid_csv_data() -> Result<(), Box<dyn Error>> {
        let mut tmp_file = NamedTempFile::new()?;
        writeln!(tmp_file, "1.0\ninvalid\n2.0\nnot_a_number\n3.0")?;
        
        let ds = DataSet::from_csv(tmp_file.path())?;
        assert_eq!(ds.count(), 3);
        assert_eq!(ds.mean(), Some(2.0));
        Ok(())
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

    pub fn process_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (line_number, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line.is_empty() {
                continue;
            }

            if self.has_header && line_number == 0 {
                continue;
            }

            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if fields.iter().any(|field| field.is_empty()) {
                return Err(format!("Empty field detected at line {}", line_number + 1).into());
            }

            records.push(fields);
        }

        if records.is_empty() {
            return Err("No valid data records found".into());
        }

        Ok(records)
    }

    pub fn validate_records(&self, records: &[Vec<String>]) -> Result<(), Box<dyn Error>> {
        if records.is_empty() {
            return Err("Empty record set".into());
        }

        let expected_len = records[0].len();
        for (idx, record) in records.iter().enumerate() {
            if record.len() != expected_len {
                return Err(format!("Record {} has {} fields, expected {}", 
                    idx + 1, record.len(), expected_len).into());
            }
        }

        Ok(())
    }

    pub fn calculate_statistics(&self, records: &[Vec<String>], column_index: usize) -> Result<(f64, f64, f64), Box<dyn Error>> {
        if column_index >= records[0].len() {
            return Err("Column index out of bounds".into());
        }

        let mut values = Vec::new();
        for record in records {
            if let Ok(value) = record[column_index].parse::<f64>() {
                values.push(value);
            } else {
                return Err(format!("Non-numeric value in column {}: {}", 
                    column_index, record[column_index]).into());
            }
        }

        if values.is_empty() {
            return Err("No numeric values found".into());
        }

        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;

        let variance: f64 = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        Ok((mean, variance, std_dev))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_file_with_header() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,score").unwrap();
        writeln!(temp_file, "Alice,25,95.5").unwrap();
        writeln!(temp_file, "Bob,30,88.0").unwrap();
        
        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0], vec!["Alice", "25", "95.5"]);
    }

    #[test]
    fn test_validate_records() {
        let records = vec![
            vec!["A".to_string(), "B".to_string()],
            vec!["C".to_string(), "D".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let result = processor.validate_records(&records);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            vec!["10.0".to_string()],
            vec!["20.0".to_string()],
            vec!["30.0".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let result = processor.calculate_statistics(&records, 0);
        
        assert!(result.is_ok());
        let (mean, variance, std_dev) = result.unwrap();
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}