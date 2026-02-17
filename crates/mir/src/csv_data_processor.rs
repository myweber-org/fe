
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Result<Self, String> {
        if name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        if category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }

        Ok(Self {
            id,
            name,
            value,
            category,
        })
    }

    pub fn transform_value(&mut self, multiplier: f64) {
        self.value *= multiplier;
    }
}

pub fn load_csv_data(file_path: &str) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut line_number = 0;

    for line in reader.lines() {
        line_number += 1;
        let line = line?;
        
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err(format!("Invalid CSV format at line {}", line_number).into());
        }

        let id = parts[0].parse::<u32>()
            .map_err(|_| format!("Invalid ID at line {}", line_number))?;
        
        let name = parts[1].trim().to_string();
        
        let value = parts[2].parse::<f64>()
            .map_err(|_| format!("Invalid value at line {}", line_number))?;
        
        let category = parts[3].trim().to_string();

        match DataRecord::new(id, name, value, category) {
            Ok(record) => records.push(record),
            Err(e) => return Err(format!("Validation error at line {}: {}", line_number, e).into()),
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[DataRecord]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = sum / count;

    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();

    (mean, variance, std_dev)
}

pub fn filter_by_category(records: Vec<DataRecord>, category: &str) -> Vec<DataRecord> {
    records.into_iter()
        .filter(|r| r.category == category)
        .collect()
}

pub fn process_data_pipeline(file_path: &str, target_category: &str, multiplier: f64) 
    -> Result<(Vec<DataRecord>, (f64, f64, f64)), Box<dyn Error>> 
{
    let mut records = load_csv_data(file_path)?;
    
    for record in records.iter_mut() {
        record.transform_value(multiplier);
    }

    let filtered_records = filter_by_category(records, target_category);
    let stats = calculate_statistics(&filtered_records);

    Ok((filtered_records, stats))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, "Test".to_string(), 10.5, "A".to_string());
        assert!(valid_record.is_ok());

        let invalid_name = DataRecord::new(2, "".to_string(), 5.0, "B".to_string());
        assert!(invalid_name.is_err());

        let invalid_value = DataRecord::new(3, "Test".to_string(), -1.0, "C".to_string());
        assert!(invalid_value.is_err());
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            DataRecord::new(1, "A".to_string(), 10.0, "X".to_string()).unwrap(),
            DataRecord::new(2, "B".to_string(), 20.0, "X".to_string()).unwrap(),
            DataRecord::new(3, "C".to_string(), 30.0, "X".to_string()).unwrap(),
        ];

        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }

    #[test]
    fn test_csv_processing() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "1,Item1,10.5,CategoryA")?;
        writeln!(temp_file, "2,Item2,15.0,CategoryB")?;
        writeln!(temp_file, "3,Item3,20.0,CategoryA")?;

        let records = load_csv_data(temp_file.path().to_str().unwrap())?;
        assert_eq!(records.len(), 3);

        let filtered = filter_by_category(records, "CategoryA");
        assert_eq!(filtered.len(), 2);

        Ok(())
    }
}