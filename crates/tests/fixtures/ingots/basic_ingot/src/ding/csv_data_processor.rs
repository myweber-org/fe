
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

    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
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

pub fn read_csv_file(file_path: &Path) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut line_number = 0;

    for line in reader.lines() {
        line_number += 1;
        let line_content = line?;
        
        if line_content.trim().is_empty() || line_content.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line_content.split(',').collect();
        if parts.len() != 4 {
            return Err(format!("Invalid format at line {}", line_number).into());
        }

        let id = parts[0].parse::<u32>()
            .map_err(|_| format!("Invalid ID at line {}", line_number))?;
        
        let name = parts[1].trim().to_string();
        let value = parts[2].parse::<f64>()
            .map_err(|_| format!("Invalid value at line {}", line_number))?;
        let category = parts[3].trim().to_string();

        let record = DataRecord::new(id, name, value, category);
        record.validate()
            .map_err(|e| format!("Validation error at line {}: {}", line_number, e))?;
        
        records.push(record);
    }

    Ok(records)
}

pub fn filter_records_by_category(records: &[DataRecord], category: &str) -> Vec<DataRecord> {
    records.iter()
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

pub fn transform_records(records: &[DataRecord]) -> Vec<DataRecord> {
    records.iter()
        .map(|r| {
            let transformed_value = if r.value > 100.0 {
                r.value * 0.9
            } else {
                r.value * 1.1
            };
            
            DataRecord::new(
                r.id,
                r.name.to_uppercase(),
                transformed_value,
                r.category.clone()
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, "Test".to_string(), 50.0, "A".to_string());
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(2, "".to_string(), -10.0, "".to_string());
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_read_csv_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Item1,100.0,CategoryA").unwrap();
        writeln!(temp_file, "2,Item2,200.0,CategoryB").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "3,Item3,300.0,CategoryA").unwrap();

        let records = read_csv_file(temp_file.path()).unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].name, "Item1");
        assert_eq!(records[1].value, 200.0);
        assert_eq!(records[2].category, "CategoryA");
    }

    #[test]
    fn test_filter_records() {
        let records = vec![
            DataRecord::new(1, "A".to_string(), 10.0, "X".to_string()),
            DataRecord::new(2, "B".to_string(), 20.0, "Y".to_string()),
            DataRecord::new(3, "C".to_string(), 30.0, "X".to_string()),
        ];

        let filtered = filter_records_by_category(&records, "X");
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, 1);
        assert_eq!(filtered[1].id, 3);
    }

    #[test]
    fn test_calculate_average() {
        let records = vec![
            DataRecord::new(1, "A".to_string(), 10.0, "X".to_string()),
            DataRecord::new(2, "B".to_string(), 20.0, "Y".to_string()),
            DataRecord::new(3, "C".to_string(), 30.0, "Z".to_string()),
        ];

        let avg = calculate_average_value(&records).unwrap();
        assert!((avg - 20.0).abs() < f64::EPSILON);

        let empty_records: Vec<DataRecord> = vec![];
        assert!(calculate_average_value(&empty_records).is_none());
    }

    #[test]
    fn test_transform_records() {
        let records = vec![
            DataRecord::new(1, "test".to_string(), 50.0, "A".to_string()),
            DataRecord::new(2, "sample".to_string(), 150.0, "B".to_string()),
        ];

        let transformed = transform_records(&records);
        assert_eq!(transformed[0].name, "TEST");
        assert!((transformed[0].value - 55.0).abs() < f64::EPSILON);
        assert!((transformed[1].value - 135.0).abs() < f64::EPSILON);
    }
}