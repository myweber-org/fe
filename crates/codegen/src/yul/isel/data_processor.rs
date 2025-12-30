use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Result<Self, String> {
        if value < 0.0 {
            return Err(format!("Invalid value {} for record {}", value, id));
        }
        if category.is_empty() {
            return Err(format!("Empty category for record {}", id));
        }
        Ok(Self { id, value, category })
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self { records: Vec::new() }
    }

    pub fn load_from_csv(&mut self, file_path: &Path) -> Result<usize, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line_num == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                continue;
            }

            let id = parts[0].parse::<u32>()?;
            let value = parts[1].parse::<f64>()?;
            let category = parts[2].to_string();

            match DataRecord::new(id, value, category) {
                Ok(record) => {
                    self.records.push(record);
                    count += 1;
                }
                Err(e) => eprintln!("Skipping line {}: {}", line_num + 1, e),
            }
        }

        Ok(count)
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn total_records(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 42.5, "test".to_string());
        assert!(record.is_ok());
        let record = record.unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "test");
    }

    #[test]
    fn test_invalid_record_creation() {
        let record = DataRecord::new(2, -10.0, "test".to_string());
        assert!(record.is_err());
        
        let record = DataRecord::new(3, 10.0, "".to_string());
        assert!(record.is_err());
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,100.5,alpha").unwrap();
        writeln!(temp_file, "2,200.0,beta").unwrap();
        writeln!(temp_file, "3,-50.0,alpha").unwrap();
        writeln!(temp_file, "4,300.0,").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
        assert_eq!(processor.total_records(), 2);
    }

    #[test]
    fn test_average_calculation() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "test".to_string()).unwrap());
        processor.records.push(DataRecord::new(2, 20.0, "test".to_string()).unwrap());
        processor.records.push(DataRecord::new(3, 30.0, "test".to_string()).unwrap());
        
        let avg = processor.calculate_average();
        assert_eq!(avg, Some(20.0));
    }

    #[test]
    fn test_empty_average() {
        let processor = DataProcessor::new();
        let avg = processor.calculate_average();
        assert_eq!(avg, None);
    }

    #[test]
    fn test_category_filter() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "alpha".to_string()).unwrap());
        processor.records.push(DataRecord::new(2, 20.0, "beta".to_string()).unwrap());
        processor.records.push(DataRecord::new(3, 30.0, "alpha".to_string()).unwrap());
        
        let alpha_records = processor.filter_by_category("alpha");
        assert_eq!(alpha_records.len(), 2);
        
        let beta_records = processor.filter_by_category("beta");
        assert_eq!(beta_records.len(), 1);
        
        let gamma_records = processor.filter_by_category("gamma");
        assert_eq!(gamma_records.len(), 0);
    }
}