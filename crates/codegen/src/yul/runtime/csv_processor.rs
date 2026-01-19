use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    delimiter: char,
    has_headers: bool,
}

impl CsvProcessor {
    pub fn new(delimiter: char, has_headers: bool) -> Self {
        CsvProcessor {
            delimiter,
            has_headers,
        }
    }

    pub fn read_file(&self, path: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines();

        if self.has_headers {
            let _ = lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            records.push(fields);
        }

        Ok(records)
    }

    pub fn filter_records<F>(&self, records: &[Vec<String>], predicate: F) -> Vec<Vec<String>>
    where
        F: Fn(&[String]) -> bool,
    {
        records
            .iter()
            .filter(|record| predicate(record))
            .cloned()
            .collect()
    }

    pub fn transform_column<F>(&self, records: &mut [Vec<String>], col_index: usize, transformer: F)
    where
        F: Fn(&str) -> String,
    {
        for record in records.iter_mut() {
            if col_index < record.len() {
                record[col_index] = transformer(&record[col_index]);
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
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        writeln!(temp_file, "Charlie,35,Tokyo").unwrap();

        let processor = CsvProcessor::new(',', true);
        let records = processor.read_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(records.len(), 3);
        assert_eq!(records[0], vec!["Alice", "30", "New York"]);

        let filtered = processor.filter_records(&records, |fields| {
            fields.get(1).and_then(|age| age.parse::<i32>().ok()).map_or(false, |age| age > 30)
        });

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0], vec!["Charlie", "35", "Tokyo"]);

        let mut transformed_records = records.clone();
        processor.transform_column(&mut transformed_records, 2, |city| city.to_uppercase());

        assert_eq!(transformed_records[0][2], "NEW YORK");
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub active: bool,
}

#[derive(Debug)]
pub enum CsvError {
    IoError(String),
    ParseError(String, usize),
    ValidationError(String),
}

impl std::fmt::Display for CsvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CsvError::IoError(msg) => write!(f, "IO error: {}", msg),
            CsvError::ParseError(msg, line) => write!(f, "Parse error at line {}: {}", line, msg),
            CsvError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for CsvError {}

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

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<CsvRecord>, CsvError> {
        let file = File::open(&path).map_err(|e| {
            CsvError::IoError(format!("Failed to open file: {}", e))
        })?;

        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut line_number = 0;

        for line in reader.lines() {
            line_number += 1;
            let line_content = line.map_err(|e| {
                CsvError::IoError(format!("Failed to read line: {}", e))
            })?;

            if line_number == 1 && self.has_header {
                continue;
            }

            if line_content.trim().is_empty() {
                continue;
            }

            let record = self.parse_line(&line_content, line_number)?;
            records.push(record);
        }

        self.validate_records(&records)?;
        Ok(records)
    }

    fn parse_line(&self, line: &str, line_number: usize) -> Result<CsvRecord, CsvError> {
        let parts: Vec<&str> = line.split(self.delimiter).collect();
        
        if parts.len() != 4 {
            return Err(CsvError::ParseError(
                format!("Expected 4 columns, found {}", parts.len()),
                line_number,
            ));
        }

        let id = parts[0].parse::<u32>().map_err(|_| {
            CsvError::ParseError(
                format!("Invalid ID format: '{}'", parts[0]),
                line_number,
            )
        })?;

        let name = parts[1].trim().to_string();
        if name.is_empty() {
            return Err(CsvError::ParseError(
                "Name cannot be empty".to_string(),
                line_number,
            ));
        }

        let value = parts[2].parse::<f64>().map_err(|_| {
            CsvError::ParseError(
                format!("Invalid value format: '{}'", parts[2]),
                line_number,
            )
        })?;

        let active = match parts[3].trim().to_lowercase().as_str() {
            "true" | "1" | "yes" => true,
            "false" | "0" | "no" => false,
            _ => return Err(CsvError::ParseError(
                format!("Invalid boolean format: '{}'", parts[3]),
                line_number,
            )),
        };

        Ok(CsvRecord {
            id,
            name,
            value,
            active,
        })
    }

    fn validate_records(&self, records: &[CsvRecord]) -> Result<(), CsvError> {
        if records.is_empty() {
            return Err(CsvError::ValidationError(
                "No valid records found in file".to_string(),
            ));
        }

        let mut seen_ids = std::collections::HashSet::new();
        for record in records {
            if !seen_ids.insert(record.id) {
                return Err(CsvError::ValidationError(
                    format!("Duplicate ID found: {}", record.id),
                ));
            }

            if record.value < 0.0 {
                return Err(CsvError::ValidationError(
                    format!("Negative value found for ID {}: {}", record.id, record.value),
                ));
            }
        }

        Ok(())
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

        (sum, mean, std_dev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let csv_content = "id,name,value,active\n1,Item1,10.5,true\n2,Item2,20.0,false\n3,Item3,15.75,yes";
        
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_content).unwrap();
        
        let processor = CsvProcessor::new(',', true);
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 3);
        
        let (sum, mean, std_dev) = CsvProcessor::calculate_statistics(&records);
        assert!((sum - 46.25).abs() < 0.001);
        assert!((mean - 15.416).abs() < 0.001);
        assert!((std_dev - 4.204).abs() < 0.01);
    }

    #[test]
    fn test_invalid_csv() {
        let csv_content = "id,name,value,active\n1,Item1,invalid,true";
        
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_content).unwrap();
        
        let processor = CsvProcessor::new(',', true);
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_err());
    }
}