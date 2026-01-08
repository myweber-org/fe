
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

impl CsvRecord {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        Self {
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

pub fn read_csv_file(file_path: &Path) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if index == 0 {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 4 {
            let id = parts[0].parse::<u32>().unwrap_or(0);
            let name = parts[1].to_string();
            let value = parts[2].parse::<f64>().unwrap_or(0.0);
            let category = parts[3].to_string();

            let record = CsvRecord::new(id, name, value, category);
            if record.is_valid() {
                records.push(record);
            }
        }
    }

    Ok(records)
}

pub fn filter_by_category(records: Vec<CsvRecord>, category: &str) -> Vec<CsvRecord> {
    records
        .into_iter()
        .filter(|record| record.category == category)
        .collect()
}

pub fn calculate_total_value(records: &[CsvRecord]) -> f64 {
    records.iter().map(|record| record.value).sum()
}

pub fn process_csv_data(file_path: &Path, target_category: &str) -> Result<f64, Box<dyn Error>> {
    let records = read_csv_file(file_path)?;
    let filtered_records = filter_by_category(records, target_category);
    
    if filtered_records.is_empty() {
        return Ok(0.0);
    }

    let total = calculate_total_value(&filtered_records);
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_record_validation() {
        let valid_record = CsvRecord::new(1, "Test".to_string(), 100.0, "A".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = CsvRecord::new(2, "".to_string(), -50.0, "".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_value_transformation() {
        let mut record = CsvRecord::new(1, "Test".to_string(), 100.0, "A".to_string());
        record.transform_value(1.5);
        assert_eq!(record.value, 150.0);
    }

    #[test]
    fn test_csv_processing() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "id,name,value,category")?;
        writeln!(temp_file, "1,Item1,100.0,Electronics")?;
        writeln!(temp_file, "2,Item2,200.0,Books")?;
        writeln!(temp_file, "3,Item3,150.0,Electronics")?;

        let total = process_csv_data(temp_file.path(), "Electronics")?;
        assert_eq!(total, 250.0);

        Ok(())
    }
}