
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

    pub fn filter_outliers(&self, threshold: f64) -> Vec<f64> {
        if let Some(mean) = self.calculate_mean() {
            if let Some(std_dev) = self.calculate_standard_deviation() {
                return self.data
                    .iter()
                    .filter(|&&x| (x - mean).abs() <= threshold * std_dev)
                    .cloned()
                    .collect();
            }
        }
        self.data.clone()
    }

    pub fn get_summary(&self) -> String {
        let mean_str = match self.calculate_mean() {
            Some(m) => format!("{:.4}", m),
            None => "N/A".to_string(),
        };
        
        let std_dev_str = match self.calculate_standard_deviation() {
            Some(s) => format!("{:.4}", s),
            None => "N/A".to_string(),
        };
        
        format!(
            "Data points: {}, Mean: {}, Std Dev: {}",
            self.data.len(),
            mean_str,
            std_dev_str
        )
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
        writeln!(temp_file, "10.5\n20.3\n15.7\n25.1\n18.9").unwrap();
        
        assert!(processor.load_from_csv(temp_file.path()).is_ok());
        assert_eq!(processor.data.len(), 5);
        
        let mean = processor.calculate_mean().unwrap();
        assert!((mean - 18.1).abs() < 0.1);
        
        let filtered = processor.filter_outliers(2.0);
        assert_eq!(filtered.len(), 5);
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
            
            if let Some(value) = self.parse_numeric_value(&line) {
                self.data.push(value);
            }
        }
        
        Ok(())
    }
    
    fn parse_header(&mut self, header_line: &str) {
        let columns: Vec<&str> = header_line.split(',').collect();
        if columns.len() >= 2 {
            self.metadata.insert("source".to_string(), columns[0].to_string());
            self.metadata.insert("unit".to_string(), columns[1].to_string());
        }
    }
    
    fn parse_numeric_value(&self, line: &str) -> Option<f64> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.is_empty() {
            return None;
        }
        
        parts[0].trim().parse::<f64>().ok()
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

#[derive(Debug, Clone, Default)]
pub struct Statistics {
    pub count: usize,
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub sum: f64,
}

impl std::fmt::Display for Statistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Statistics: count={}, mean={:.2}, std_dev={:.2}, min={:.2}, max={:.2}",
               self.count, self.mean, self.std_dev, self.min, self.max)
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
        writeln!(temp_file, "temperature,celsius").unwrap();
        writeln!(temp_file, "23.5").unwrap();
        writeln!(temp_file, "24.1").unwrap();
        writeln!(temp_file, "22.8").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.data_count(), 3);
        
        let stats = processor.calculate_statistics();
        assert_eq!(stats.count, 3);
        assert!((stats.mean - 23.466).abs() < 0.001);
        
        let filtered = processor.filter_by_threshold(23.0);
        assert_eq!(filtered.len(), 2);
        
        let metadata = processor.get_metadata();
        assert_eq!(metadata.get("unit"), Some(&"celsius".to_string()));
    }
}