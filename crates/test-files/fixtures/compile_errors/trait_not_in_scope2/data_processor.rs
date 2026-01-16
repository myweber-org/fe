
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u32, values: Vec<f64>) -> Self {
        Self {
            id,
            values,
            metadata: HashMap::new(),
        }
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        if self.id == 0 {
            return Err("Invalid record ID".into());
        }
        
        if self.values.is_empty() {
            return Err("Empty values vector".into());
        }

        for value in &self.values {
            if value.is_nan() || value.is_infinite() {
                return Err("Invalid numeric value detected".into());
            }
        }

        Ok(())
    }

    pub fn normalize(&mut self) {
        if let Some(max) = self.values.iter().copied().reduce(f64::max) {
            if max != 0.0 {
                for value in &mut self.values {
                    *value /= max;
                }
            }
        }
    }

    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        let count = self.values.len() as f64;
        let sum: f64 = self.values.iter().sum();
        let mean = sum / count;

        let variance: f64 = self.values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let mut processed = Vec::new();

    for record in records {
        record.validate()?;
        let mut processed_record = record.clone();
        processed_record.normalize();
        processed.push(processed_record);
    }

    Ok(processed)
}

pub fn aggregate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();
    let mut total_mean = 0.0;
    let mut total_variance = 0.0;
    let mut count = 0;

    for record in records {
        let (mean, variance, std_dev) = record.calculate_statistics();
        total_mean += mean;
        total_variance += variance;
        count += 1;

        stats.insert(format!("record_{}_mean", record.id), mean);
        stats.insert(format!("record_{}_std_dev", record.id), std_dev);
    }

    if count > 0 {
        stats.insert("overall_mean".to_string(), total_mean / count as f64);
        stats.insert("overall_variance".to_string(), total_variance / count as f64);
    }

    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, vec![1.0, 2.0, 3.0]);
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(0, vec![1.0, 2.0]);
        assert!(invalid_record.validate().is_err());

        let nan_record = DataRecord::new(2, vec![1.0, f64::NAN]);
        assert!(nan_record.validate().is_err());
    }

    #[test]
    fn test_normalization() {
        let mut record = DataRecord::new(1, vec![1.0, 2.0, 3.0]);
        record.normalize();
        assert_eq!(record.values, vec![1.0/3.0, 2.0/3.0, 1.0]);
    }

    #[test]
    fn test_statistics_calculation() {
        let record = DataRecord::new(1, vec![1.0, 2.0, 3.0]);
        let (mean, variance, std_dev) = record.calculate_statistics();
        
        assert!((mean - 2.0).abs() < 1e-10);
        assert!((variance - 2.0/3.0).abs() < 1e-10);
        assert!((std_dev - (2.0/3.0 as f64).sqrt()).abs() < 1e-10);
    }
}