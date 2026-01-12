
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataRecord {
    id: u32,
    value: f64,
    timestamp: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, timestamp: &str) -> Result<Self, String> {
        if value < 0.0 {
            return Err("Value cannot be negative".to_string());
        }
        if timestamp.is_empty() {
            return Err("Timestamp cannot be empty".to_string());
        }
        Ok(Self {
            id,
            value,
            timestamp: timestamp.to_string(),
        })
    }

    pub fn calculate_adjusted_value(&self, multiplier: f64) -> f64 {
        self.value * multiplier
    }
}

pub fn load_records_from_csv(file_path: &Path) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = csv::Reader::from_reader(file);
    let mut records = Vec::new();

    for result in rdr.deserialize() {
        let raw_record: (u32, f64, String) = result?;
        match DataRecord::new(raw_record.0, raw_record.1, &raw_record.2) {
            Ok(record) => records.push(record),
            Err(e) => eprintln!("Skipping invalid record: {}", e),
        }
    }

    Ok(records)
}

pub fn process_records(records: &[DataRecord]) -> (f64, f64, usize) {
    let count = records.len();
    if count == 0 {
        return (0.0, 0.0, 0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let avg = sum / count as f64;
    let max = records
        .iter()
        .map(|r| r.value)
        .fold(f64::NEG_INFINITY, f64::max);

    (avg, max, count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_record_creation() {
        let record = DataRecord::new(1, 42.5, "2024-01-15T10:30:00Z");
        assert!(record.is_ok());
        let record = record.unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.timestamp, "2024-01-15T10:30:00Z");
    }

    #[test]
    fn test_invalid_record_creation() {
        let record = DataRecord::new(1, -5.0, "2024-01-15T10:30:00Z");
        assert!(record.is_err());
        
        let record = DataRecord::new(1, 5.0, "");
        assert!(record.is_err());
    }

    #[test]
    fn test_calculate_adjusted_value() {
        let record = DataRecord::new(1, 10.0, "2024-01-15T10:30:00Z").unwrap();
        assert_eq!(record.calculate_adjusted_value(2.5), 25.0);
    }

    #[test]
    fn test_load_and_process_records() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "id,value,timestamp")?;
        writeln!(temp_file, "1,10.5,2024-01-15T10:30:00Z")?;
        writeln!(temp_file, "2,20.0,2024-01-15T11:30:00Z")?;
        writeln!(temp_file, "3,15.5,2024-01-15T12:30:00Z")?;

        let records = load_records_from_csv(temp_file.path())?;
        assert_eq!(records.len(), 3);

        let (avg, max, count) = process_records(&records);
        assert_eq!(count, 3);
        assert!((avg - 15.333333333333334).abs() < 0.0001);
        assert_eq!(max, 20.0);

        Ok(())
    }
}