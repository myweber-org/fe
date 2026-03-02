use std::error::Error;
use std::fs::File;
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }

        Ok(())
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

    pub fn validate_records(&self) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.value >= 0.0 && r.value <= 1000.0)
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
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,100.5,TypeA").unwrap();
        writeln!(temp_file, "2,200.3,TypeB").unwrap();
        writeln!(temp_file, "3,150.7,TypeA").unwrap();

        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.records.len(), 3);

        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 150.5).abs() < 0.1);

        let filtered = processor.filter_by_category("TypeA");
        assert_eq!(filtered.len(), 2);

        let stats = processor.get_statistics();
        assert!((stats.0 - 100.5).abs() < 0.1);
        assert!((stats.1 - 200.3).abs() < 0.1);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
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
            
            let category = parts[2].trim();
            
            match DataRecord::new(id, value, category) {
                Ok(record) => {
                    self.records.push(record);
                    count += 1;
                }
                Err(_) => continue,
            }
        }
        
        Ok(count)
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
    
    pub fn get_record_count(&self) -> usize {
        self.records.len()
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
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.get_record_count(), 0);
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,category_a").unwrap();
        writeln!(temp_file, "2,20.0,category_b").unwrap();
        writeln!(temp_file, "3,15.75,category_a").unwrap();
        
        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);
        assert_eq!(processor.get_record_count(), 3);
        
        let filtered = processor.filter_by_category("category_a");
        assert_eq!(filtered.len(), 2);
        
        let total = processor.calculate_total_value();
        assert_eq!(total, 46.25);
        
        let average = processor.get_average_value().unwrap();
        assert!((average - 15.416666666666666).abs() < 0.0001);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct DataProcessor {
    file_path: String,
}

impl DataProcessor {
    pub fn new(file_path: &str) -> Self {
        DataProcessor {
            file_path: file_path.to_string(),
        }
    }

    pub fn process(&self) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            let fields: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();

            if fields.len() < 2 {
                return Err(format!("Invalid data format at line {}", index + 1).into());
            }

            records.push(fields);
        }

        if records.is_empty() {
            return Err("No data found in file".into());
        }

        Ok(records)
    }

    pub fn calculate_average(&self, column_index: usize) -> Result<f64, Box<dyn Error>> {
        let records = self.process()?;
        let mut sum = 0.0;
        let mut count = 0;

        for record in records.iter().skip(1) {
            if let Some(value) = record.get(column_index) {
                if let Ok(num) = value.parse::<f64>() {
                    sum += num;
                    count += 1;
                }
            }
        }

        if count == 0 {
            return Err("No valid numeric data found".into());
        }

        Ok(sum / count as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,score").unwrap();
        writeln!(temp_file, "Alice,25,95.5").unwrap();
        writeln!(temp_file, "Bob,30,87.0").unwrap();

        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let result = processor.process().unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["name", "age", "score"]);
    }

    #[test]
    fn test_average_calculation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,score").unwrap();
        writeln!(temp_file, "Alice,95.5").unwrap();
        writeln!(temp_file, "Bob,87.0").unwrap();
        writeln!(temp_file, "Charlie,92.5").unwrap();

        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let average = processor.calculate_average(1).unwrap();

        assert!((average - 91.6667).abs() < 0.001);
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

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines();

        if self.has_header {
            lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !fields.is_empty() {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn validate_record(&self, record: &[String]) -> bool {
        !record.is_empty() && record.iter().all(|field| !field.is_empty())
    }

    pub fn extract_column(&self, data: &[Vec<String>], column_index: usize) -> Vec<String> {
        data.iter()
            .filter_map(|record| record.get(column_index).cloned())
            .collect()
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
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "30", "New York"]);
        assert_eq!(result[1], vec!["Bob", "25", "London"]);
    }

    #[test]
    fn test_validation() {
        let processor = DataProcessor::new(',', false);
        let valid_record = vec!["data".to_string(), "value".to_string()];
        let invalid_record = vec!["".to_string(), "value".to_string()];

        assert!(processor.validate_record(&valid_record));
        assert!(!processor.validate_record(&invalid_record));
    }

    #[test]
    fn test_column_extraction() {
        let data = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string(), "d".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let column = processor.extract_column(&data, 1);
        
        assert_eq!(column, vec!["b".to_string(), "d".to_string()]);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    data: Vec<f64>,
    metadata: HashMap<String, String>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
                self.parse_header(&line);
                continue;
            }
            
            if let Some(value) = self.parse_value(&line) {
                self.data.push(value);
            }
        }
        
        Ok(())
    }

    fn parse_header(&mut self, header_line: &str) {
        let parts: Vec<&str> = header_line.split(',').collect();
        if parts.len() >= 2 {
            self.metadata.insert("source".to_string(), parts[0].to_string());
            self.metadata.insert("unit".to_string(), parts[1].to_string());
        }
    }

    fn parse_value(&self, line: &str) -> Option<f64> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.is_empty() {
            return None;
        }
        
        parts[0].trim().parse().ok()
    }

    pub fn calculate_statistics(&self) -> Statistics {
        if self.data.is_empty() {
            return Statistics::default();
        }

        let sum: f64 = self.data.iter().sum();
        let count = self.data.len();
        let mean = sum / count as f64;
        
        let variance: f64 = self.data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count as f64;
        
        let std_dev = variance.sqrt();
        
        let min = self.data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        Statistics {
            count,
            mean,
            std_dev,
            min,
            max,
            sum,
        }
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<f64> {
        self.data.iter()
            .filter(|&&x| x >= threshold)
            .cloned()
            .collect()
    }

    pub fn get_metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    pub fn data_count(&self) -> usize {
        self.data.len()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Statistics {
    pub count: usize,
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub sum: f64,
}

impl Default for Statistics {
    fn default() -> Self {
        Statistics {
            count: 0,
            mean: 0.0,
            std_dev: 0.0,
            min: 0.0,
            max: 0.0,
            sum: 0.0,
        }
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
        writeln!(temp_file, "test_data,unit").unwrap();
        writeln!(temp_file, "10.5").unwrap();
        writeln!(temp_file, "20.3").unwrap();
        writeln!(temp_file, "15.7").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.data_count(), 3);
        
        let stats = processor.calculate_statistics();
        assert_eq!(stats.count, 3);
        assert!((stats.mean - 15.5).abs() < 0.1);
        
        let filtered = processor.filter_by_threshold(15.0);
        assert_eq!(filtered.len(), 2);
        
        let metadata = processor.get_metadata();
        assert_eq!(metadata.get("source").unwrap(), "test_data");
    }
}