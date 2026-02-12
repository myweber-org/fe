
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct DataSet {
    values: Vec<f64>,
    mean: Option<f64>,
    variance: Option<f64>,
}

impl DataSet {
    pub fn new() -> Self {
        DataSet {
            values: Vec::new(),
            mean: None,
            variance: None,
        }
    }

    pub fn from_csv<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut dataset = DataSet::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(value) = line.trim().parse::<f64>() {
                dataset.values.push(value);
            }
        }

        Ok(dataset)
    }

    pub fn add_value(&mut self, value: f64) {
        self.values.push(value);
        self.mean = None;
        self.variance = None;
    }

    pub fn calculate_mean(&mut self) -> f64 {
        if let Some(mean) = self.mean {
            return mean;
        }

        if self.values.is_empty() {
            self.mean = Some(0.0);
            return 0.0;
        }

        let sum: f64 = self.values.iter().sum();
        let mean = sum / self.values.len() as f64;
        self.mean = Some(mean);
        mean
    }

    pub fn calculate_variance(&mut self) -> f64 {
        if let Some(variance) = self.variance {
            return variance;
        }

        if self.values.len() < 2 {
            self.variance = Some(0.0);
            return 0.0;
        }

        let mean = self.calculate_mean();
        let sum_squared_diff: f64 = self.values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum();
        
        let variance = sum_squared_diff / (self.values.len() - 1) as f64;
        self.variance = Some(variance);
        variance
    }

    pub fn get_values(&self) -> &[f64] {
        &self.values
    }

    pub fn clear(&mut self) {
        self.values.clear();
        self.mean = None;
        self.variance = None;
    }
}

pub fn filter_outliers(data: &[f64], threshold: f64) -> Vec<f64> {
    if data.len() < 3 {
        return data.to_vec();
    }

    let mut temp_dataset = DataSet::new();
    for &value in data {
        temp_dataset.add_value(value);
    }

    let mean = temp_dataset.calculate_mean();
    let std_dev = temp_dataset.calculate_variance().sqrt();

    data.iter()
        .filter(|&&x| (x - mean).abs() <= threshold * std_dev)
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_dataset_operations() {
        let mut dataset = DataSet::new();
        dataset.add_value(10.0);
        dataset.add_value(20.0);
        dataset.add_value(30.0);

        assert_eq!(dataset.calculate_mean(), 20.0);
        assert_eq!(dataset.calculate_variance(), 100.0);
    }

    #[test]
    fn test_csv_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "10.5\n20.3\n30.7\ninvalid\n40.1").unwrap();
        
        let dataset = DataSet::from_csv(temp_file.path()).unwrap();
        assert_eq!(dataset.get_values(), &[10.5, 20.3, 30.7, 40.1]);
    }

    #[test]
    fn test_outlier_filtering() {
        let data = vec![1.0, 2.0, 3.0, 100.0];
        let filtered = filter_outliers(&data, 2.0);
        assert_eq!(filtered, vec![1.0, 2.0, 3.0]);
    }
}