use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    data: Vec<f64>,
    frequency_map: HashMap<String, u32>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: Vec::new(),
            frequency_map: HashMap::new(),
        }
    }

    pub fn load_numeric_data(&mut self, filepath: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if let Ok(value) = line.trim().parse::<f64>() {
                self.data.push(value);
            }
        }
        Ok(())
    }

    pub fn load_categorical_data(&mut self, filepath: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(filepath)?;
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

    pub fn get_frequency_distribution(&self) -> Vec<(&String, &u32)> {
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
    fn test_statistics_calculation() {
        let mut processor = DataProcessor::new();
        processor.data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        let (mean, median, variance, std_dev) = processor.calculate_statistics();
        
        assert_eq!(mean, 3.0);
        assert_eq!(median, 3.0);
        assert_eq!(variance, 2.0);
        assert!((std_dev - 1.4142135623730951).abs() < 1e-10);
    }

    #[test]
    fn test_data_filtering() {
        let mut processor = DataProcessor::new();
        processor.data = vec![1.0, 5.0, 3.0, 8.0, 2.0];
        
        let filtered = processor.filter_data(3.0);
        assert_eq!(filtered, vec![5.0, 8.0]);
    }

    #[test]
    fn test_file_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "10.5\n20.3\n15.7\n").unwrap();
        
        let mut processor = DataProcessor::new();
        let result = processor.load_numeric_data(temp_file.path().to_str().unwrap());
        
        assert!(result.is_ok());
        assert_eq!(processor.data.len(), 3);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Self {
        DataRecord { id, value, category }
    }

    pub fn is_valid(&self) -> bool {
        self.id > 0 && self.value >= 0.0 && !self.category.is_empty()
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { records: Vec::new() }
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
                Ok(val) => val,
                Err(_) => continue,
            };

            let value = match parts[1].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let record = DataRecord::new(id, value, parts[2].to_string());
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
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

    pub fn total_records(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, 10.5, "A".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(0, -5.0, "".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,TypeA").unwrap();
        writeln!(temp_file, "2,15.3,TypeB").unwrap();
        writeln!(temp_file, "invalid,data,row").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(processor.total_records(), 2);
    }

    #[test]
    fn test_average_calculation() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "A".to_string()));
        processor.records.push(DataRecord::new(2, 20.0, "B".to_string()));
        
        assert_eq!(processor.calculate_average(), Some(15.0));
        assert_eq!(DataProcessor::new().calculate_average(), None);
    }
}
use std::collections::HashMap;

pub struct DataProcessor {
    cache: HashMap<String, Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            cache: HashMap::new(),
        }
    }

    pub fn process_dataset(&mut self, key: &str, data: &[f64]) -> Result<Vec<f64>, String> {
        if data.is_empty() {
            return Err("Empty dataset provided".to_string());
        }

        if let Some(cached) = self.cache.get(key) {
            return Ok(cached.clone());
        }

        let processed = Self::normalize_data(data);
        self.cache.insert(key.to_string(), processed.clone());
        
        Ok(processed)
    }

    fn normalize_data(data: &[f64]) -> Vec<f64> {
        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let variance = data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64;
        let std_dev = variance.sqrt();

        if std_dev.abs() < 1e-10 {
            return vec![0.0; data.len()];
        }

        data.iter()
            .map(|&x| (x - mean) / std_dev)
            .collect()
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn get_cache_stats(&self) -> (usize, usize) {
        let total_items: usize = self.cache.values().map(|v| v.len()).sum();
        (self.cache.len(), total_items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_data() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let normalized = DataProcessor::normalize_data(&data);
        
        let sum: f64 = normalized.iter().sum();
        let sum_sq: f64 = normalized.iter().map(|x| x * x).sum();
        
        assert!(sum.abs() < 1e-10);
        assert!((sum_sq - (data.len() as f64 - 1.0)).abs() < 1e-10);
    }

    #[test]
    fn test_empty_dataset() {
        let mut processor = DataProcessor::new();
        let result = processor.process_dataset("test", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_cache_functionality() {
        let mut processor = DataProcessor::new();
        let data = vec![10.0, 20.0, 30.0];
        
        let first_result = processor.process_dataset("dataset1", &data).unwrap();
        let second_result = processor.process_dataset("dataset1", &data).unwrap();
        
        assert_eq!(first_result, second_result);
        assert_eq!(processor.get_cache_stats(), (1, 1));
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub timestamp: i64,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String, timestamp: i64) -> Self {
        DataRecord {
            id,
            value,
            category,
            timestamp,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.id > 0 && self.value.is_finite() && !self.category.is_empty() && self.timestamp > 0
    }
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
        let mut loaded_count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                eprintln!("Warning: Invalid format at line {}", line_num + 1);
                continue;
            }

            let id = match parts[0].parse::<u32>() {
                Ok(val) => val,
                Err(_) => {
                    eprintln!("Warning: Invalid ID at line {}", line_num + 1);
                    continue;
                }
            };

            let value = match parts[1].parse::<f64>() {
                Ok(val) => val,
                Err(_) => {
                    eprintln!("Warning: Invalid value at line {}", line_num + 1);
                    continue;
                }
            };

            let category = parts[2].trim().to_string();
            if category.is_empty() {
                eprintln!("Warning: Empty category at line {}", line_num + 1);
                continue;
            }

            let timestamp = match parts[3].parse::<i64>() {
                Ok(val) => val,
                Err(_) => {
                    eprintln!("Warning: Invalid timestamp at line {}", line_num + 1);
                    continue;
                }
            };

            let record = DataRecord::new(id, value, category, timestamp);
            if record.is_valid() {
                self.records.push(record);
                loaded_count += 1;
            } else {
                eprintln!("Warning: Invalid record at line {}", line_num + 1);
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

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
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

    pub fn count_records(&self) -> usize {
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
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, 42.5, "test".to_string(), 1234567890);
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(0, 42.5, "test".to_string(), 1234567890);
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,42.5,category_a,1234567890").unwrap();
        writeln!(temp_file, "2,99.9,category_b,1234567891").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
        assert_eq!(processor.count_records(), 2);
    }

    #[test]
    fn test_filtering() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "A".to_string(), 1000));
        processor.records.push(DataRecord::new(2, 20.0, "B".to_string(), 2000));
        processor.records.push(DataRecord::new(3, 30.0, "A".to_string(), 3000));

        let filtered = processor.filter_by_category("A");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_statistics() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "test".to_string(), 1000));
        processor.records.push(DataRecord::new(2, 20.0, "test".to_string(), 2000));
        processor.records.push(DataRecord::new(3, 30.0, "test".to_string(), 3000));

        let (min, max, avg) = processor.get_statistics();
        assert_eq!(min, 10.0);
        assert_eq!(max, 30.0);
        assert_eq!(avg, 20.0);
    }
}