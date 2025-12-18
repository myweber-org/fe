
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
        let mut validated = Vec::with_capacity(data.len());
        
        for &value in data {
            if value.is_nan() || value.is_infinite() {
                return Err(format!("Invalid numeric value detected: {}", value));
            }
            validated.push(value);
        }
        
        Ok(validated)
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
    fn test_validation_rejects_invalid_data() {
        let processor = DataProcessor::new();
        let invalid_data = [1.0, f64::NAN, 3.0];
        
        assert!(processor.validate_data(&invalid_data).is_err());
    }

    #[test]
    fn test_normalization_produces_zero_mean() {
        let processor = DataProcessor::new();
        let data = [1.0, 2.0, 3.0, 4.0, 5.0];
        let normalized = processor.normalize_data(&data);
        
        let mean = normalized.iter().sum::<f64>() / normalized.len() as f64;
        assert!(mean.abs() < 1e-10);
    }

    #[test]
    fn test_cache_functionality() {
        let mut processor = DataProcessor::new();
        let data = [1.5, 2.5, 3.5];
        
        let result1 = processor.process_dataset("test", &data).unwrap();
        let result2 = processor.process_dataset("test", &data).unwrap();
        
        assert_eq!(result1, result2);
        assert_eq!(processor.cache_stats(), (1, 3));
    }
}use std::error::Error;
use std::fs::File;
use std::path::Path;

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
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        
        let mut records = Vec::new();
        for result in rdr.records() {
            let record = result?;
            let row: Vec<String> = record.iter().map(|field| field.to_string()).collect();
            
            if self.validate_row(&row) {
                records.push(row);
            } else {
                eprintln!("Warning: Skipping invalid row: {:?}", row);
            }
        }
        
        Ok(records)
    }

    fn validate_row(&self, row: &[String]) -> bool {
        !row.is_empty() && row.iter().all(|field| !field.trim().is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,30,New York").unwrap();
        writeln!(temp_file, "Alice,25,London").unwrap();
        
        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let result = processor.process().unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["John", "30", "New York"]);
        assert_eq!(result[1], vec!["Alice", "25", "London"]);
    }

    #[test]
    fn test_invalid_data_skipping() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "field1,field2").unwrap();
        writeln!(temp_file, "value1,").unwrap();
        writeln!(temp_file, ",value2").unwrap();
        writeln!(temp_file, "valid1,valid2").unwrap();
        
        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let result = processor.process().unwrap();
        
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], vec!["valid1", "valid2"]);
    }
}