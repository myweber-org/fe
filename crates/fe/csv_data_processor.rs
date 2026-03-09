
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        DataRecord {
            id,
            name,
            value,
            category,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0 && !self.category.is_empty()
    }

    pub fn transform_value(&mut self, multiplier: f64) {
        self.value *= multiplier;
    }
}

pub fn load_csv_data(file_path: &Path) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut line_number = 0;

    for line in reader.lines() {
        line_number += 1;
        let line = line?;
        
        if line_number == 1 {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            continue;
        }

        let id = match parts[0].parse::<u32>() {
            Ok(val) => val,
            Err(_) => continue,
        };

        let name = parts[1].to_string();
        let value = match parts[2].parse::<f64>() {
            Ok(val) => val,
            Err(_) => continue,
        };

        let category = parts[3].to_string();

        let record = DataRecord::new(id, name, value, category);
        if record.is_valid() {
            records.push(record);
        }
    }

    Ok(records)
}

pub fn filter_by_category(records: &[DataRecord], category: &str) -> Vec<DataRecord> {
    records
        .iter()
        .filter(|r| r.category == category)
        .cloned()
        .collect()
}

pub fn calculate_average_value(records: &[DataRecord]) -> Option<f64> {
    if records.is_empty() {
        return None;
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    Some(sum / records.len() as f64)
}

pub fn process_data_pipeline(
    file_path: &Path,
    target_category: &str,
    value_multiplier: f64,
) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let mut records = load_csv_data(file_path)?;
    
    for record in records.iter_mut() {
        record.transform_value(value_multiplier);
    }

    let filtered_records = filter_by_category(&records, target_category);
    Ok(filtered_records)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, "Test".to_string(), 10.5, "A".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = DataRecord::new(2, "".to_string(), -5.0, "".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_value_transformation() {
        let mut record = DataRecord::new(1, "Test".to_string(), 10.0, "A".to_string());
        record.transform_value(2.0);
        assert_eq!(record.value, 20.0);
    }

    #[test]
    fn test_csv_loading() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "id,name,value,category")?;
        writeln!(temp_file, "1,Item1,10.5,CategoryA")?;
        writeln!(temp_file, "2,Item2,20.0,CategoryB")?;
        
        let records = load_csv_data(temp_file.path())?;
        assert_eq!(records.len(), 2);
        Ok(())
    }

    #[test]
    fn test_category_filtering() {
        let records = vec![
            DataRecord::new(1, "A".to_string(), 10.0, "Cat1".to_string()),
            DataRecord::new(2, "B".to_string(), 20.0, "Cat2".to_string()),
            DataRecord::new(3, "C".to_string(), 30.0, "Cat1".to_string()),
        ];

        let filtered = filter_by_category(&records, "Cat1");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_average_calculation() {
        let records = vec![
            DataRecord::new(1, "A".to_string(), 10.0, "Cat1".to_string()),
            DataRecord::new(2, "B".to_string(), 20.0, "Cat2".to_string()),
            DataRecord::new(3, "C".to_string(), 30.0, "Cat1".to_string()),
        ];

        let avg = calculate_average_value(&records);
        assert_eq!(avg, Some(20.0));

        let empty_avg = calculate_average_value(&[]);
        assert_eq!(empty_avg, None);
    }
}