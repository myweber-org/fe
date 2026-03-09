use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

#[derive(Debug)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    fn from_csv_line(line: &str) -> Result<Self, Box<dyn Error>> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err("Invalid number of fields".into());
        }

        let id = parts[0].parse()?;
        let name = parts[1].to_string();
        let value = parts[2].parse()?;
        let active = parts[3].parse()?;

        Ok(Record {
            id,
            name,
            value,
            active,
        })
    }

    fn to_csv_line(&self) -> String {
        format!("{},{},{},{}", self.id, self.name, self.value, self.active)
    }
}

fn read_records_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        match Record::from_csv_line(&line) {
            Ok(record) => records.push(record),
            Err(e) => eprintln!("Warning: Skipping line {}: {}", line_num + 1, e),
        }
    }

    Ok(records)
}

fn filter_records(records: &[Record], min_value: f64) -> Vec<&Record> {
    records
        .iter()
        .filter(|r| r.value >= min_value && r.active)
        .collect()
}

fn write_records_to_file<P: AsRef<Path>>(
    records: &[&Record],
    path: P,
) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(path)?;
    for record in records {
        writeln!(file, "{}", record.to_csv_line())?;
    }
    Ok(())
}

fn process_csv_file(input_path: &str, output_path: &str, threshold: f64) -> Result<(), Box<dyn Error>> {
    println!("Processing CSV file: {}", input_path);
    
    let records = read_records_from_file(input_path)?;
    println!("Loaded {} records", records.len());
    
    let filtered = filter_records(&records, threshold);
    println!("Found {} records meeting criteria (value >= {}, active=true)", 
             filtered.len(), threshold);
    
    write_records_to_file(&filtered, output_path)?;
    println!("Results written to: {}", output_path);
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/filtered.csv";
    let threshold = 50.0;
    
    process_csv_file(input_file, output_file, threshold)
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            headers: Vec::new(),
            records: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        if let Some(first_line) = lines.next() {
            self.headers = first_line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
        }

        for line in lines {
            let record: Vec<String> = line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            if record.len() == self.headers.len() {
                self.records.push(record);
            }
        }

        Ok(())
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

    pub fn get_column_names(&self) -> &[String] {
        &self.headers
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        writeln!(temp_file, "Charlie,30,Paris").unwrap();

        let mut processor = CsvProcessor::new();
        processor.load_from_file(temp_file.path()).unwrap();

        assert_eq!(processor.record_count(), 3);
        assert_eq!(processor.get_column_names(), &["name", "age", "city"]);

        let filtered = processor.filter_by_column("age", "30");
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0][0], "Alice");
        assert_eq!(filtered[1][0], "Charlie");
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub enum CsvError {
    IoError(std::io::Error),
    ParseError(String),
    ValidationError(String),
}

impl From<std::io::Error> for CsvError {
    fn from(err: std::io::Error) -> Self {
        CsvError::IoError(err)
    }
}

pub struct CsvProcessor {
    delimiter: char,
    has_header: bool,
}

impl CsvProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        CsvProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, CsvError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut line_number = 0;

        for line in reader.lines() {
            line_number += 1;
            let line_content = line?;
            
            if line_content.trim().is_empty() {
                continue;
            }

            let record: Vec<String> = line_content
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if record.is_empty() {
                return Err(CsvError::ParseError(
                    format!("Empty record at line {}", line_number)
                ));
            }

            if self.has_header && line_number == 1 {
                continue;
            }

            records.push(record);
        }

        if records.is_empty() {
            return Err(CsvError::ValidationError(
                "No valid data records found".to_string()
            ));
        }

        Ok(records)
    }

    pub fn validate_records(&self, records: &[Vec<String>]) -> Result<(), CsvError> {
        if records.is_empty() {
            return Err(CsvError::ValidationError(
                "Empty records vector".to_string()
            ));
        }

        let expected_columns = records[0].len();
        
        for (idx, record) in records.iter().enumerate() {
            if record.len() != expected_columns {
                return Err(CsvError::ValidationError(
                    format!("Record {} has {} columns, expected {}", 
                           idx + 1, record.len(), expected_columns)
                ));
            }
            
            for (col_idx, field) in record.iter().enumerate() {
                if field.is_empty() {
                    return Err(CsvError::ValidationError(
                        format!("Empty field at record {}, column {}", 
                               idx + 1, col_idx + 1)
                    ));
                }
            }
        }

        Ok(())
    }
}

pub fn calculate_column_average(records: &[Vec<String>], column_index: usize) -> Result<f64, CsvError> {
    if records.is_empty() {
        return Err(CsvError::ValidationError(
            "Cannot calculate average from empty records".to_string()
        ));
    }

    if column_index >= records[0].len() {
        return Err(CsvError::ValidationError(
            format!("Column index {} out of bounds", column_index)
        ));
    }

    let mut sum = 0.0;
    let mut count = 0;

    for record in records {
        if let Ok(value) = record[column_index].parse::<f64>() {
            sum += value;
            count += 1;
        } else {
            return Err(CsvError::ParseError(
                format!("Failed to parse numeric value at column {}", column_index)
            ));
        }
    }

    if count == 0 {
        return Err(CsvError::ValidationError(
            "No valid numeric values found in specified column".to_string()
        ));
    }

    Ok(sum / count as f64)
}