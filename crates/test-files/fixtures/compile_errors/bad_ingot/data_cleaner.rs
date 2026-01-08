use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    age: u8,
    active: bool,
}

fn clean_data(input_path: &str, output_path: &str, min_age: u8) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(input_file);

    let output_file = File::create(output_path)?;
    let mut writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(output_file);

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.age >= min_age && record.active {
            writer.serialize(&record)?;
        }
    }

    writer.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    clean_data("input.csv", "cleaned.csv", 18)?;
    println!("Data cleaning completed successfully");
    Ok(())
}use std::collections::HashMap;

pub struct DataCleaner {
    data: Vec<f64>,
    thresholds: HashMap<String, f64>,
}

impl DataCleaner {
    pub fn new(data: Vec<f64>) -> Self {
        DataCleaner {
            data,
            thresholds: HashMap::new(),
        }
    }

    pub fn calculate_iqr_thresholds(&mut self) {
        let mut sorted_data = self.data.clone();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let q1_index = (sorted_data.len() as f64 * 0.25) as usize;
        let q3_index = (sorted_data.len() as f64 * 0.75) as usize;

        let q1 = sorted_data[q1_index];
        let q3 = sorted_data[q3_index];
        let iqr = q3 - q1;

        self.thresholds.insert("lower".to_string(), q1 - 1.5 * iqr);
        self.thresholds.insert("upper".to_string(), q3 + 1.5 * iqr);
    }

    pub fn remove_outliers(&self) -> Vec<f64> {
        let lower = *self.thresholds.get("lower").unwrap_or(&f64::MIN);
        let upper = *self.thresholds.get("upper").unwrap_or(&f64::MAX);

        self.data
            .iter()
            .filter(|&&value| value >= lower && value <= upper)
            .cloned()
            .collect()
    }

    pub fn get_thresholds(&self) -> &HashMap<String, f64> {
        &self.thresholds
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outlier_removal() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 100.0];
        let mut cleaner = DataCleaner::new(data);
        cleaner.calculate_iqr_thresholds();
        let cleaned = cleaner.remove_outliers();
        
        assert_eq!(cleaned.len(), 5);
        assert!(!cleaned.contains(&100.0));
    }
}