use std::error::Error;
use std::fs::File;
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
        let mut rdr = csv::Reader::from_reader(file);
        let mut values = Vec::new();

        for result in rdr.records() {
            let record = result?;
            if let Some(field) = record.get(0) {
                if let Ok(num) = field.parse::<f64>() {
                    values.push(num);
                }
            }
        }

        Ok(DataSet { values })
    }

    pub fn add_value(&mut self, value: f64) {
        self.values.push(value);
    }

    pub fn calculate_mean(&self) -> Option<f64> {
        if self.values.is_empty() {
            return None;
        }
        let sum: f64 = self.values.iter().sum();
        Some(sum / self.values.len() as f64)
    }

    pub fn calculate_std_dev(&self) -> Option<f64> {
        if self.values.len() < 2 {
            return None;
        }
        let mean = self.calculate_mean()?;
        let variance: f64 = self.values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / (self.values.len() - 1) as f64;
        Some(variance.sqrt())
    }

    pub fn count(&self) -> usize {
        self.values.len()
    }

    pub fn clear(&mut self) {
        self.values.clear();
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
        assert_eq!(ds.count(), 0);
        assert_eq!(ds.calculate_mean(), None);
        assert_eq!(ds.calculate_std_dev(), None);
    }

    #[test]
    fn test_basic_statistics() {
        let mut ds = DataSet::new();
        ds.add_value(10.0);
        ds.add_value(20.0);
        ds.add_value(30.0);
        
        assert_eq!(ds.count(), 3);
        assert_eq!(ds.calculate_mean(), Some(20.0));
        assert!(ds.calculate_std_dev().unwrap() > 8.16 && ds.calculate_std_dev().unwrap() < 8.17);
    }

    #[test]
    fn test_csv_import() -> Result<(), Box<dyn Error>> {
        let mut tmp_file = NamedTempFile::new()?;
        writeln!(tmp_file, "value\n15.5\n25.3\n35.7")?;
        
        let ds = DataSet::from_csv(tmp_file.path())?;
        assert_eq!(ds.count(), 3);
        assert!(ds.calculate_mean().unwrap() > 25.49 && ds.calculate_mean().unwrap() < 25.51);
        Ok(())
    }

    #[test]
    fn test_clear_operation() {
        let mut ds = DataSet::new();
        ds.add_value(5.0);
        ds.add_value(15.0);
        assert_eq!(ds.count(), 2);
        
        ds.clear();
        assert_eq!(ds.count(), 0);
        assert_eq!(ds.calculate_mean(), None);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct DataProcessor {
    data: Vec<f64>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { data: Vec::new() }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
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

    pub fn get_data_summary(&self) -> (usize, Option<f64>, Option<f64>) {
        (
            self.data.len(),
            self.calculate_mean(),
            self.calculate_standard_deviation()
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
        writeln!(temp_file, "10.5\n15.2\n12.8\n14.1\n13.7").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        let (count, mean, std_dev) = processor.get_data_summary();
        
        assert_eq!(count, 5);
        assert!(mean.is_some());
        assert!(std_dev.is_some());
        
        let mean_val = mean.unwrap();
        let std_dev_val = std_dev.unwrap();
        
        assert!((mean_val - 13.26).abs() < 0.01);
        assert!((std_dev_val - 1.75).abs() < 0.01);
    }
}