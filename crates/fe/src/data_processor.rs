
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    file_path: String,
    delimiter: char,
}

impl DataProcessor {
    pub fn new(file_path: &str) -> Self {
        DataProcessor {
            file_path: file_path.to_string(),
            delimiter: ',',
        }
    }

    pub fn with_delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = delimiter;
        self
    }

    pub fn process(&self) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut records = Vec::new();
        for line in reader.lines() {
            let line = line?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !fields.is_empty() && !fields.iter().all(|f| f.is_empty()) {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn filter_by_column(&self, column_index: usize, predicate: impl Fn(&str) -> bool) 
        -> Result<Vec<Vec<String>>, Box<dyn Error>> 
    {
        let records = self.process()?;
        let filtered: Vec<Vec<String>> = records
            .into_iter()
            .filter(|record| {
                if column_index < record.len() {
                    predicate(&record[column_index])
                } else {
                    false
                }
            })
            .collect();

        Ok(filtered)
    }

    pub fn get_column_stats(&self, column_index: usize) -> Result<ColumnStats, Box<dyn Error>> {
        let records = self.process()?;
        let mut numeric_values = Vec::new();
        let mut text_values = Vec::new();

        for record in records {
            if column_index < record.len() {
                let value = &record[column_index];
                if let Ok(num) = value.parse::<f64>() {
                    numeric_values.push(num);
                } else {
                    text_values.push(value.clone());
                }
            }
        }

        Ok(ColumnStats {
            numeric_count: numeric_values.len(),
            text_count: text_values.len(),
            numeric_sum: numeric_values.iter().sum(),
            numeric_avg: if !numeric_values.is_empty() {
                numeric_values.iter().sum::<f64>() / numeric_values.len() as f64
            } else {
                0.0
            },
        })
    }
}

pub struct ColumnStats {
    pub numeric_count: usize,
    pub text_count: usize,
    pub numeric_sum: f64,
    pub numeric_avg: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        writeln!(temp_file, "Charlie,35,Paris").unwrap();

        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let result = processor.process().unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0][0], "Alice");
        assert_eq!(result[1][1], "25");
    }

    #[test]
    fn test_filter_by_column() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        writeln!(temp_file, "Charlie,35,Paris").unwrap();

        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let filtered = processor.filter_by_column(1, |age| {
            age.parse::<i32>().map_or(false, |a| a >= 30)
        }).unwrap();

        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0][0], "Alice");
        assert_eq!(filtered[1][0], "Charlie");
    }
}use csv::Reader;
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
        validate_record(&record)?;
        records.push(record);
    }

    Ok(records)
}

fn validate_record(record: &Record) -> Result<(), String> {
    if record.name.is_empty() {
        return Err("Name cannot be empty".to_string());
    }
    if record.value < 0.0 {
        return Err("Value must be non-negative".to_string());
    }
    if !["A", "B", "C"].contains(&record.category.as_str()) {
        return Err("Category must be A, B, or C".to_string());
    }
    Ok(())
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
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

pub fn filter_by_category(records: Vec<Record>, category: &str) -> Vec<Record> {
    records.into_iter()
        .filter(|r| r.category == category)
        .collect()
}