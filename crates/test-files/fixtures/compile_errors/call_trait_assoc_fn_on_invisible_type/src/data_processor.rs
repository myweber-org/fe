
use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn process_data_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
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

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
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

pub fn filter_by_category(records: Vec<Record>, category: &str) -> Vec<Record> {
    records.into_iter()
        .filter(|r| r.category == category)
        .collect()
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct DataProcessor {
    pub valid_records: Vec<String>,
    pub invalid_records: Vec<String>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            valid_records: Vec::new(),
            invalid_records: Vec::new(),
        }
    }

    pub fn process_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);

        for (line_num, line) in reader.lines().enumerate() {
            let record = line?;
            
            if self.validate_record(&record) {
                self.valid_records.push(record);
            } else {
                self.invalid_records.push(format!("Line {}: {}", line_num + 1, record));
            }
        }

        Ok(())
    }

    fn validate_record(&self, record: &str) -> bool {
        let fields: Vec<&str> = record.split(',').collect();
        
        if fields.len() != 3 {
            return false;
        }

        fields.iter().all(|field| !field.trim().is_empty())
    }

    pub fn get_statistics(&self) -> (usize, usize) {
        (self.valid_records.len(), self.invalid_records.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "John,Doe,30").unwrap();
        writeln!(temp_file, "Jane,Smith,25").unwrap();
        writeln!(temp_file, "Invalid,Record").unwrap();
        writeln!(temp_file, ",,").unwrap();

        let result = processor.process_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());

        let (valid, invalid) = processor.get_statistics();
        assert_eq!(valid, 2);
        assert_eq!(invalid, 2);
        assert_eq!(processor.valid_records.len(), 2);
        assert_eq!(processor.invalid_records.len(), 2);
    }
}