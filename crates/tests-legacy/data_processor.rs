
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
}