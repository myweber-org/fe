
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
    valid: bool,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Self {
        let valid = value >= 0.0 && value <= 1000.0;
        DataRecord {
            id,
            value,
            category,
            valid,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn get_value(&self) -> f64 {
        self.value
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    total_value: f64,
    valid_count: usize,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            total_value: 0.0,
            valid_count: 0,
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line_num == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                continue;
            }

            let id = match parts[0].parse::<u32>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let value = match parts[1].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let category = parts[2].to_string();

            let record = DataRecord::new(id, value, category);
            self.add_record(record);
        }

        Ok(())
    }

    pub fn add_record(&mut self, record: DataRecord) {
        if record.is_valid() {
            self.total_value += record.get_value();
            self.valid_count += 1;
        }
        self.records.push(record);
    }

    pub fn get_average_value(&self) -> Option<f64> {
        if self.valid_count > 0 {
            Some(self.total_value / self.valid_count as f64)
        } else {
            None
        }
    }

    pub fn get_valid_records(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| r.is_valid()).collect()
    }

    pub fn count_by_category(&self) -> std::collections::HashMap<String, usize> {
        let mut counts = std::collections::HashMap::new();
        for record in &self.records {
            if record.is_valid() {
                *counts.entry(record.category.clone()).or_insert(0) += 1;
            }
        }
        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 500.0, "A".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(2, -10.0, "B".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_processor_average() {
        let mut processor = DataProcessor::new();
        processor.add_record(DataRecord::new(1, 100.0, "Test".to_string()));
        processor.add_record(DataRecord::new(2, 200.0, "Test".to_string()));
        processor.add_record(DataRecord::new(3, -50.0, "Test".to_string()));

        assert_eq!(processor.get_average_value(), Some(150.0));
        assert_eq!(processor.valid_count, 2);
    }
}