
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
            for field in record.iter() {
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

    pub fn filter_outliers(&self, threshold: f64) -> Vec<f64> {
        let mean = match self.calculate_mean() {
            Some(m) => m,
            None => return Vec::new(),
        };
        let std_dev = match self.calculate_std_dev() {
            Some(s) => s,
            None => return Vec::new(),
        };

        self.values
            .iter()
            .filter(|&&x| (x - mean).abs() <= threshold * std_dev)
            .cloned()
            .collect()
    }

    pub fn get_summary(&self) -> String {
        format!(
            "Count: {}, Mean: {:.2}, StdDev: {:.2}",
            self.values.len(),
            self.calculate_mean().unwrap_or(0.0),
            self.calculate_std_dev().unwrap_or(0.0)
        )
    }
}

pub fn process_numeric_data(data: &[f64]) -> (f64, f64, f64) {
    let min = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let range = max - min;
    (min, max, range)
}