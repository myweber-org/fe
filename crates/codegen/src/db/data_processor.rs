use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
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

            let category = parts[2].trim().to_string();
            if category.is_empty() {
                continue;
            }

            self.records.push(DataRecord { id, value, category });
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

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn get_records(&self) -> &[DataRecord] {
        &self.records
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
        writeln!(temp_file, "1,10.5,alpha").unwrap();
        writeln!(temp_file, "2,20.3,beta").unwrap();
        writeln!(temp_file, "3,15.7,alpha").unwrap();

        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);

        let alpha_records = processor.filter_by_category("alpha");
        assert_eq!(alpha_records.len(), 2);

        let avg = processor.calculate_average().unwrap();
        assert!((avg - 15.5).abs() < 0.01);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    data: Vec<f64>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { data: Vec::new() }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        for line in reader.lines() {
            let line = line?;
            if let Ok(value) = line.trim().parse::<f64>() {
                self.data.push(value);
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

    pub fn calculate_standard_deviation(&self) -> Option<f64> {
        if self.data.len() < 2 {
            return None;
        }
        
        let mean = self.calculate_mean()?;
        let variance: f64 = self.data
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / (self.data.len() - 1) as f64;
        
        Some(variance.sqrt())
    }

    pub fn find_outliers(&self, threshold: f64) -> Vec<(usize, f64)> {
        let mean = match self.calculate_mean() {
            Some(m) => m,
            None => return Vec::new(),
        };
        
        let std_dev = match self.calculate_standard_deviation() {
            Some(s) => s,
            None => return Vec::new(),
        };
        
        self.data
            .iter()
            .enumerate()
            .filter(|(_, &value)| (value - mean).abs() > threshold * std_dev)
            .map(|(idx, &value)| (idx, value))
            .collect()
    }

    pub fn add_data_point(&mut self, value: f64) {
        self.data.push(value);
    }

    pub fn clear_data(&mut self) {
        self.data.clear();
    }

    pub fn data_count(&self) -> usize {
        self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_empty_processor() {
        let processor = DataProcessor::new();
        assert_eq!(processor.data_count(), 0);
        assert_eq!(processor.calculate_mean(), None);
        assert_eq!(processor.calculate_standard_deviation(), None);
    }

    #[test]
    fn test_basic_statistics() {
        let mut processor = DataProcessor::new();
        processor.add_data_point(10.0);
        processor.add_data_point(20.0);
        processor.add_data_point(30.0);
        
        assert_eq!(processor.calculate_mean(), Some(20.0));
        assert!(processor.calculate_standard_deviation().unwrap() > 0.0);
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "10.5\n20.3\n30.7\n40.1").unwrap();
        
        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        
        assert!(result.is_ok());
        assert_eq!(processor.data_count(), 4);
        assert_eq!(processor.calculate_mean(), Some(25.4));
    }

    #[test]
    fn test_outlier_detection() {
        let mut processor = DataProcessor::new();
        processor.add_data_point(10.0);
        processor.add_data_point(11.0);
        processor.add_data_point(12.0);
        processor.add_data_point(100.0); // Outlier
        
        let outliers = processor.find_outliers(2.0);
        assert_eq!(outliers.len(), 1);
        assert_eq!(outliers[0].0, 3);
        assert_eq!(outliers[0].1, 100.0);
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
        let mut lines = reader.lines().enumerate();

        if self.has_header {
            lines.next();
        }

        for (line_number, line_result) in lines {
            let line = line_result?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if fields.iter().any(|f| f.is_empty()) {
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
        let expected_len = records.first().map(|r| r.len()).unwrap_or(0);
        
        for (idx, record) in records.iter().enumerate() {
            if record.len() != expected_len {
                return Err(format!("Record {} has {} fields, expected {}", 
                    idx + 1, record.len(), expected_len).into());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_valid_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,30,New York").unwrap();
        writeln!(temp_file, "Alice,25,London").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0], vec!["John", "30", "New York"]);
    }

    #[test]
    fn test_empty_field_error() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,,New York").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Empty field"));
    }
}