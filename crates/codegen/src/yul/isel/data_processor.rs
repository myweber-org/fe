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
            if line.trim().is_empty() {
                continue;
            }
            let value: f64 = line.trim().parse()?;
            self.data.push(value);
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
        if let Some(std_dev) = self.calculate_standard_deviation() {
            if let Some(mean) = self.calculate_mean() {
                let lower_bound = mean - threshold * std_dev;
                let upper_bound = mean + threshold * std_dev;
                return self.data
                    .iter()
                    .filter(|&&x| x >= lower_bound && x <= upper_bound)
                    .copied()
                    .collect();
            }
        }
        self.data.clone()
    }

    pub fn get_summary(&self) -> String {
        let mean = self.calculate_mean().unwrap_or(0.0);
        let std_dev = self.calculate_standard_deviation().unwrap_or(0.0);
        let min = self.data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        format!(
            "Statistics Summary:\n  Count: {}\n  Mean: {:.4}\n  Std Dev: {:.4}\n  Min: {:.4}\n  Max: {:.4}",
            self.data.len(),
            mean,
            std_dev,
            min,
            max
        )
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
        writeln!(temp_file, "10.5\n20.3\n15.7\n25.1\n18.9").unwrap();
        
        processor.load_from_csv(temp_file.path()).unwrap();
        
        assert_eq!(processor.data.len(), 5);
        assert!(processor.calculate_mean().unwrap() - 18.1 < 0.001);
        assert!(processor.calculate_standard_deviation().unwrap() - 5.5 < 0.1);
        
        let filtered = processor.filter_outliers(1.5);
        assert!(filtered.len() > 0);
        
        let summary = processor.get_summary();
        assert!(summary.contains("Count: 5"));
    }
}