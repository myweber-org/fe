
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataRecord {
    id: u32,
    value: f64,
    timestamp: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, timestamp: &str) -> Self {
        DataRecord {
            id,
            value,
            timestamp: timestamp.to_string(),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0 {
            return Err("ID cannot be zero".to_string());
        }
        if self.value < 0.0 || self.value > 1000.0 {
            return Err("Value must be between 0 and 1000".to_string());
        }
        if self.timestamp.is_empty() {
            return Err("Timestamp cannot be empty".to_string());
        }
        Ok(())
    }
}

pub fn process_csv_file(file_path: &str) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let path = Path::new(file_path);
    let file = File::open(path)?;
    let mut rdr = csv::Reader::from_reader(file);
    
    let mut records = Vec::new();
    
    for result in rdr.deserialize() {
        let record: DataRecord = result?;
        record.validate()?;
        records.push(record);
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, 500.0, "2024-01-15T10:30:00Z");
        assert!(valid_record.validate().is_ok());
        
        let invalid_record = DataRecord::new(0, 500.0, "2024-01-15T10:30:00Z");
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_process_csv_file() {
        let csv_data = "id,value,timestamp\n1,100.5,2024-01-15T10:30:00Z\n2,200.3,2024-01-15T11:30:00Z\n";
        
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_data).unwrap();
        
        let result = process_csv_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            DataRecord::new(1, 100.0, "2024-01-15T10:30:00Z"),
            DataRecord::new(2, 200.0, "2024-01-15T11:30:00Z"),
            DataRecord::new(3, 300.0, "2024-01-15T12:30:00Z"),
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 200.0);
        assert_eq!(variance, 6666.666666666667);
        assert_eq!(std_dev, 81.64965809277261);
    }
}