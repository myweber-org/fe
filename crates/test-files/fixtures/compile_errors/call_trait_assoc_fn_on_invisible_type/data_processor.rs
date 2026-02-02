
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub valid: bool,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str) -> Self {
        let valid = value >= 0.0 && !category.is_empty();
        DataRecord {
            id,
            value,
            category: category.to_string(),
            valid,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0 {
            return Err("ID cannot be zero".to_string());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        if self.category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(())
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) {
        self.records.push(record);
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut line_count = 0;

        for line in reader.lines() {
            line_count += 1;
            let line = line?;
            
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                eprintln!("Warning: Line {} has invalid format, skipping", line_count);
                continue;
            }

            let id = match parts[0].parse::<u32>() {
                Ok(id) => id,
                Err(_) => {
                    eprintln!("Warning: Invalid ID on line {}, skipping", line_count);
                    continue;
                }
            };

            let value = match parts[1].parse::<f64>() {
                Ok(value) => value,
                Err(_) => {
                    eprintln!("Warning: Invalid value on line {}, skipping", line_count);
                    continue;
                }
            };

            let category = parts[2].trim();
            let record = DataRecord::new(id, value, category);
            
            if let Err(e) = record.validate() {
                eprintln!("Warning: Invalid record on line {}: {}, skipping", line_count, e);
                continue;
            }

            self.add_record(record);
        }

        Ok(())
    }

    pub fn filter_valid(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| r.valid).collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records: Vec<&DataRecord> = self.filter_valid();
        
        if valid_records.is_empty() {
            return None;
        }

        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&DataRecord>> {
        let mut groups = std::collections::HashMap::new();
        
        for record in &self.records {
            if record.valid {
                groups
                    .entry(record.category.clone())
                    .or_insert_with(Vec::new)
                    .push(record);
            }
        }
        
        groups
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn count_valid(&self) -> usize {
        self.filter_valid().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, 42.5, "test");
        assert!(valid_record.validate().is_ok());
        assert!(valid_record.valid);

        let invalid_record = DataRecord::new(0, -1.0, "");
        assert!(invalid_record.validate().is_err());
        assert!(!invalid_record.valid);
    }

    #[test]
    fn test_load_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,10.5,category_a").unwrap();
        writeln!(temp_file, "2,20.0,category_b").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "3,invalid,category_c").unwrap();

        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path());
        
        assert!(result.is_ok());
        assert_eq!(processor.count_records(), 2);
        assert_eq!(processor.count_valid(), 2);
    }

    #[test]
    fn test_average_calculation() {
        let mut processor = DataProcessor::new();
        processor.add_record(DataRecord::new(1, 10.0, "cat1"));
        processor.add_record(DataRecord::new(2, 20.0, "cat2"));
        processor.add_record(DataRecord::new(3, 30.0, "cat1"));

        let average = processor.calculate_average();
        assert_eq!(average, Some(20.0));
    }

    #[test]
    fn test_empty_processor() {
        let processor = DataProcessor::new();
        assert_eq!(processor.count_records(), 0);
        assert_eq!(processor.calculate_average(), None);
    }

    #[test]
    fn test_grouping() {
        let mut processor = DataProcessor::new();
        processor.add_record(DataRecord::new(1, 10.0, "alpha"));
        processor.add_record(DataRecord::new(2, 20.0, "beta"));
        processor.add_record(DataRecord::new(3, 30.0, "alpha"));

        let groups = processor.group_by_category();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups.get("alpha").unwrap().len(), 2);
        assert_eq!(groups.get("beta").unwrap().len(), 1);
    }
}