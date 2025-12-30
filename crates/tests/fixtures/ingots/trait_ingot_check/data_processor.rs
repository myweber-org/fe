
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

        let validated = self.validate_data(data)?;
        let normalized = self.normalize_data(&validated);
        let transformed = self.apply_transformations(&normalized);

        self.cache.insert(key.to_string(), transformed.clone());
        Ok(transformed)
    }

    fn validate_data(&self, data: &[f64]) -> Result<Vec<f64>, String> {
        let mut result = Vec::with_capacity(data.len());
        
        for &value in data {
            if !value.is_finite() {
                return Err("Invalid numeric value detected".to_string());
            }
            result.push(value);
        }
        
        Ok(result)
    }

    fn normalize_data(&self, data: &[f64]) -> Vec<f64> {
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

    fn apply_transformations(&self, data: &[f64]) -> Vec<f64> {
        data.iter()
            .map(|&x| x.powi(2).ln_1p())
            .collect()
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn cache_stats(&self) -> (usize, usize) {
        let total_items: usize = self.cache.values().map(|v| v.len()).sum();
        (self.cache.len(), total_items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_dataset() {
        let mut processor = DataProcessor::new();
        let result = processor.process_dataset("test", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_processing() {
        let mut processor = DataProcessor::new();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        let result = processor.process_dataset("valid", &data);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed.len(), data.len());
    }

    #[test]
    fn test_cache_functionality() {
        let mut processor = DataProcessor::new();
        let data = vec![10.0, 20.0, 30.0];
        
        let first_result = processor.process_dataset("cached", &data).unwrap();
        let second_result = processor.process_dataset("cached", &data).unwrap();
        
        assert_eq!(first_result, second_result);
        
        let stats = processor.cache_stats();
        assert_eq!(stats.0, 1);
        assert_eq!(stats.1, 3);
    }
}
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

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        let mut count = 0;

        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            self.validate_record(&record)?;
            self.records.push(record);
            count += 1;
        }

        Ok(count)
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), String> {
        if record.id == 0 {
            return Err("ID cannot be zero".to_string());
        }
        if record.value.is_nan() || record.value.is_infinite() {
            return Err("Value must be a finite number".to_string());
        }
        if record.category.trim().is_empty() {
            return Err("Category cannot be empty".to_string());
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

    pub fn get_statistics(&self) -> Statistics {
        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        let count = values.len();

        if count == 0 {
            return Statistics::empty();
        }

        let sum: f64 = values.iter().sum();
        let avg = sum / count as f64;
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        let variance: f64 = values.iter().map(|v| (v - avg).powi(2)).sum::<f64>() / count as f64;
        let std_dev = variance.sqrt();

        Statistics {
            count,
            sum,
            average: avg,
            min,
            max,
            standard_deviation: std_dev,
        }
    }
}

pub struct Statistics {
    pub count: usize,
    pub sum: f64,
    pub average: f64,
    pub min: f64,
    pub max: f64,
    pub standard_deviation: f64,
}

impl Statistics {
    fn empty() -> Self {
        Statistics {
            count: 0,
            sum: 0.0,
            average: 0.0,
            min: 0.0,
            max: 0.0,
            standard_deviation: 0.0,
        }
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
        writeln!(temp_file, "1,10.5,alpha").unwrap();
        writeln!(temp_file, "2,20.3,beta").unwrap();
        writeln!(temp_file, "3,15.7,alpha").unwrap();

        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);

        let avg = processor.calculate_average().unwrap();
        assert!((avg - 15.5).abs() < 0.001);

        let alpha_records = processor.filter_by_category("alpha");
        assert_eq!(alpha_records.len(), 2);

        let stats = processor.get_statistics();
        assert_eq!(stats.count, 3);
        assert!((stats.min - 10.5).abs() < 0.001);
        assert!((stats.max - 20.3).abs() < 0.001);
    }
}