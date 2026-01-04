
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

    pub fn calculate_variance(&self) -> Option<f64> {
        if self.values.len() < 2 {
            return None;
        }
        let mean = self.calculate_mean()?;
        let sum_sq_diff: f64 = self.values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum();
        Some(sum_sq_diff / (self.values.len() - 1) as f64)
    }

    pub fn calculate_standard_deviation(&self) -> Option<f64> {
        self.calculate_variance().map(|v| v.sqrt())
    }

    pub fn get_summary(&self) -> Summary {
        Summary {
            count: self.values.len(),
            mean: self.calculate_mean(),
            variance: self.calculate_variance(),
            std_dev: self.calculate_standard_deviation(),
        }
    }
}

pub struct Summary {
    pub count: usize,
    pub mean: Option<f64>,
    pub variance: Option<f64>,
    pub std_dev: Option<f64>,
}

impl std::fmt::Display for Summary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Data Summary:")?;
        writeln!(f, "  Count: {}", self.count)?;
        if let Some(mean) = self.mean {
            writeln!(f, "  Mean: {:.4}", mean)?;
        }
        if let Some(variance) = self.variance {
            writeln!(f, "  Variance: {:.4}", variance)?;
        }
        if let Some(std_dev) = self.std_dev {
            writeln!(f, "  Standard Deviation: {:.4}", std_dev)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_empty_dataset() {
        let dataset = DataSet::new();
        assert_eq!(dataset.calculate_mean(), None);
        assert_eq!(dataset.calculate_variance(), None);
        assert_eq!(dataset.calculate_standard_deviation(), None);
    }

    #[test]
    fn test_basic_statistics() {
        let mut dataset = DataSet::new();
        dataset.add_value(1.0);
        dataset.add_value(2.0);
        dataset.add_value(3.0);
        dataset.add_value(4.0);
        dataset.add_value(5.0);

        assert_eq!(dataset.calculate_mean(), Some(3.0));
        assert_eq!(dataset.calculate_variance(), Some(2.5));
        assert_eq!(dataset.calculate_standard_deviation(), Some(2.5_f64.sqrt()));
    }

    #[test]
    fn test_csv_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1.5,2.5,3.5\n4.5,5.5,6.5").unwrap();
        
        let dataset = DataSet::from_csv(temp_file.path()).unwrap();
        assert_eq!(dataset.values.len(), 6);
        assert_eq!(dataset.values[0], 1.5);
        assert_eq!(dataset.values[5], 6.5);
    }

    #[test]
    fn test_summary_display() {
        let mut dataset = DataSet::new();
        dataset.add_value(10.0);
        dataset.add_value(20.0);
        
        let summary = dataset.get_summary();
        let display_output = format!("{}", summary);
        assert!(display_output.contains("Data Summary:"));
        assert!(display_output.contains("Count: 2"));
        assert!(display_output.contains("Mean: 15.0000"));
    }
}