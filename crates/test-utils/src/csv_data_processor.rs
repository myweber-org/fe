use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

impl CsvRecord {
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

    pub fn to_json(&self) -> String {
        format!(
            r#"{{"id":{},"name":"{}","value":{},"category":"{}"}}"#,
            self.id, self.name, self.value, self.category
        )
    }
}

pub fn process_csv_file(file_path: &str) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
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

        match CsvRecord::new(id, name, value, category) {
            Ok(record) => records.push(record),
            Err(e) => eprintln!("Warning: Skipping line {}: {}", line_number, e),
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[CsvRecord]) -> (f64, f64, f64) {
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

pub fn filter_by_category(records: Vec<CsvRecord>, category: &str) -> Vec<CsvRecord> {
    records.into_iter()
        .filter(|r| r.category == category)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_record_creation() {
        let record = CsvRecord::new(1, "Test".to_string(), 100.0, "A".to_string());
        assert!(record.is_ok());
        
        let invalid_record = CsvRecord::new(2, "".to_string(), -10.0, "".to_string());
        assert!(invalid_record.is_err());
    }

    #[test]
    fn test_value_transformation() {
        let mut record = CsvRecord::new(1, "Test".to_string(), 100.0, "A".to_string()).unwrap();
        record.transform_value(1.5);
        assert_eq!(record.value, 150.0);
    }

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Item1,100.5,CategoryA").unwrap();
        writeln!(temp_file, "2,Item2,200.0,CategoryB").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();
        
        let result = process_csv_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            CsvRecord::new(1, "A".to_string(), 10.0, "X".to_string()).unwrap(),
            CsvRecord::new(2, "B".to_string(), 20.0, "X".to_string()).unwrap(),
            CsvRecord::new(3, "C".to_string(), 30.0, "X".to_string()).unwrap(),
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }

    #[test]
    fn test_category_filter() {
        let records = vec![
            CsvRecord::new(1, "A".to_string(), 10.0, "X".to_string()).unwrap(),
            CsvRecord::new(2, "B".to_string(), 20.0, "Y".to_string()).unwrap(),
            CsvRecord::new(3, "C".to_string(), 30.0, "X".to_string()).unwrap(),
        ];
        
        let filtered = filter_by_category(records, "X");
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.category == "X"));
    }
}