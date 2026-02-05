
use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    age: u8,
    active: bool,
}

fn filter_and_transform(input_path: &str, output_path: &str, min_age: u8) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = Reader::from_reader(input_file);
    
    let output_file = File::create(output_path)?;
    let mut writer = Writer::from_writer(output_file);

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.age >= min_age && record.active {
            let transformed_record = Record {
                name: record.name.to_uppercase(),
                ..record
            };
            writer.serialize(transformed_record)?;
        }
    }
    
    writer.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/output.csv";
    let minimum_age = 25;
    
    filter_and_transform(input_file, output_file, minimum_age)?;
    
    println!("Processing completed successfully");
    Ok(())
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn new(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let headers = match lines.next() {
            Some(header_line) => header_line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            None => return Err("Empty CSV file".into()),
        };

        let mut records = Vec::new();
        for line in lines {
            let line = line?;
            let record: Vec<String> = line
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            if record.len() == headers.len() {
                records.push(record);
            }
        }

        Ok(CsvProcessor { headers, records })
    }

    pub fn filter_by_column(&self, column_name: &str, value: &str) -> Vec<Vec<String>> {
        let column_index = match self.headers.iter().position(|h| h == column_name) {
            Some(idx) => idx,
            None => return Vec::new(),
        };

        self.records
            .iter()
            .filter(|record| record.get(column_index) == Some(&value.to_string()))
            .cloned()
            .collect()
    }

    pub fn get_column_stats(&self, column_name: &str) -> Option<(usize, usize, f64)> {
        let column_index = self.headers.iter().position(|h| h == column_name)?;
        
        let numeric_values: Vec<f64> = self.records
            .iter()
            .filter_map(|record| record.get(column_index)?.parse::<f64>().ok())
            .collect();

        if numeric_values.is_empty() {
            return None;
        }

        let count = numeric_values.len();
        let min = numeric_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = numeric_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg = numeric_values.iter().sum::<f64>() / count as f64;

        Some((count, min as usize, max as usize, avg))
    }

    pub fn display_summary(&self) {
        println!("CSV Summary:");
        println!("Headers: {}", self.headers.join(", "));
        println!("Total records: {}", self.records.len());
        
        for header in &self.headers {
            if let Some((count, min, max, avg)) = self.get_column_stats(header) {
                println!("  {}: {} numeric values, min={}, max={}, avg={:.2}", 
                        header, count, min, max, avg);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let csv_data = "name,age,salary\nAlice,30,50000\nBob,25,45000\nAlice,35,60000";
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_data).unwrap();
        
        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(processor.headers, vec!["name", "age", "salary"]);
        assert_eq!(processor.records.len(), 3);
        
        let alice_records = processor.filter_by_column("name", "Alice");
        assert_eq!(alice_records.len(), 2);
        
        let stats = processor.get_column_stats("age").unwrap();
        assert_eq!(stats.0, 3);
        assert_eq!(stats.1, 25);
        assert_eq!(stats.2, 35);
    }
}