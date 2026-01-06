use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

fn process_data(input_path: &str, output_path: &str, min_value: f64) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut writer = Writer::from_path(output_path)?;

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.value >= min_value && record.active {
            writer.serialize(&record)?;
        }
    }

    writer.flush()?;
    Ok(())
}

fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    let count = records.len() as f64;
    if count == 0.0 {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (mean, variance, std_dev)
}

fn validate_record(record: &Record) -> bool {
    !record.name.is_empty() && record.id > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            Record { id: 1, name: "Test1".to_string(), value: 10.0, active: true },
            Record { id: 2, name: "Test2".to_string(), value: 20.0, active: true },
            Record { id: 3, name: "Test3".to_string(), value: 30.0, active: true },
        ];
        
        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }

    #[test]
    fn test_record_validation() {
        let valid_record = Record {
            id: 1,
            name: "Valid".to_string(),
            value: 100.0,
            active: true,
        };
        
        let invalid_record = Record {
            id: 0,
            name: "".to_string(),
            value: 50.0,
            active: false,
        };
        
        assert!(validate_record(&valid_record));
        assert!(!validate_record(&invalid_record));
    }
}
use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

pub fn process_csv_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        if record.value >= 0.0 {
            records.push(record);
        }
    }

    Ok(records)
}

pub fn calculate_average(records: &[Record]) -> f64 {
    if records.is_empty() {
        return 0.0;
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    sum / records.len() as f64
}

pub fn filter_active_records(records: Vec<Record>) -> Vec<Record> {
    records.into_iter().filter(|r| r.active).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_process_csv() {
        let csv_data = "id,name,value,active\n1,Test1,10.5,true\n2,Test2,-5.0,false\n";
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_data).unwrap();
        
        let result = process_csv_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].name, "Test1");
    }

    #[test]
    fn test_calculate_average() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, active: true },
            Record { id: 2, name: "B".to_string(), value: 20.0, active: true },
        ];
        assert_eq!(calculate_average(&records), 15.0);
    }
}