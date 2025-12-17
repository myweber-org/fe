
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Result<Self, &'static str> {
        if value < 0.0 {
            return Err("Value cannot be negative");
        }
        if category.is_empty() {
            return Err("Category cannot be empty");
        }
        Ok(Self { id, value, category })
    }

    pub fn calculate_adjusted_value(&self, multiplier: f64) -> f64 {
        self.value * multiplier
    }
}

pub fn load_records_from_csv(file_path: &str) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let path = Path::new(file_path);
    let file = File::open(path)?;
    let mut rdr = csv::Reader::from_reader(file);
    let mut records = Vec::new();

    for result in rdr.deserialize() {
        let (id, value, category): (u32, f64, String) = result?;
        match DataRecord::new(id, value, category) {
            Ok(record) => records.push(record),
            Err(e) => eprintln!("Skipping invalid record: {}", e),
        }
    }

    Ok(records)
}

pub fn process_data_set(records: &[DataRecord]) -> (f64, f64, usize) {
    if records.is_empty() {
        return (0.0, 0.0, 0);
    }

    let total: f64 = records.iter().map(|r| r.value).sum();
    let average = total / records.len() as f64;
    let max_value = records.iter().map(|r| r.value).fold(0.0, f64::max);
    let unique_categories = records.iter().map(|r| &r.category).collect::<std::collections::HashSet<_>>().len();

    (average, max_value, unique_categories)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 42.5, String::from("A")).unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "A");
    }

    #[test]
    fn test_invalid_negative_value() {
        let result = DataRecord::new(2, -10.0, String::from("B"));
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_adjusted_value() {
        let record = DataRecord::new(3, 100.0, String::from("C")).unwrap();
        assert_eq!(record.calculate_adjusted_value(1.5), 150.0);
    }

    #[test]
    fn test_process_empty_dataset() {
        let records: Vec<DataRecord> = Vec::new();
        let (avg, max, categories) = process_data_set(&records);
        assert_eq!(avg, 0.0);
        assert_eq!(max, 0.0);
        assert_eq!(categories, 0);
    }
}