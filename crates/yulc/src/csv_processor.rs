use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
enum CsvError {
    IoError(std::io::Error),
    ParseError(String, usize),
    InvalidHeader(String),
}

impl fmt::Display for CsvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CsvError::IoError(e) => write!(f, "IO error: {}", e),
            CsvError::ParseError(msg, line) => write!(f, "Parse error at line {}: {}", line, msg),
            CsvError::InvalidHeader(msg) => write!(f, "Invalid header: {}", msg),
        }
    }
}

impl Error for CsvError {}

impl From<std::io::Error> for CsvError {
    fn from(error: std::io::Error) -> Self {
        CsvError::IoError(error)
    }
}

struct CsvProcessor {
    delimiter: char,
    expected_columns: usize,
}

impl CsvProcessor {
    fn new(delimiter: char, expected_columns: usize) -> Self {
        CsvProcessor {
            delimiter,
            expected_columns,
        }
    }

    fn process_file(&self, path: &str) -> Result<Vec<Vec<String>>, CsvError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (line_num, line_result) in reader.lines().enumerate() {
            let line = line_result?;
            let record = self.parse_line(&line, line_num + 1)?;
            records.push(record);
        }

        if !records.is_empty() {
            self.validate_header(&records[0])?;
        }

        Ok(records)
    }

    fn parse_line(&self, line: &str, line_num: usize) -> Result<Vec<String>, CsvError> {
        let fields: Vec<String> = line
            .split(self.delimiter)
            .map(|s| s.trim().to_string())
            .collect();

        if fields.len() != self.expected_columns {
            return Err(CsvError::ParseError(
                format!(
                    "Expected {} columns, found {}",
                    self.expected_columns,
                    fields.len()
                ),
                line_num,
            ));
        }

        Ok(fields)
    }

    fn validate_header(&self, header: &[String]) -> Result<(), CsvError> {
        for (idx, field) in header.iter().enumerate() {
            if field.is_empty() {
                return Err(CsvError::InvalidHeader(format!(
                    "Empty header field at position {}",
                    idx + 1
                )));
            }
        }
        Ok(())
    }

    fn extract_column(&self, records: &[Vec<String>], column_index: usize) -> Vec<String> {
        records
            .iter()
            .skip(1)
            .filter_map(|record| record.get(column_index).cloned())
            .collect()
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let processor = CsvProcessor::new(',', 3);
    
    match processor.process_file("data.csv") {
        Ok(records) => {
            println!("Successfully processed {} records", records.len());
            
            if records.len() > 1 {
                let first_column = processor.extract_column(&records, 0);
                println!("First column values: {:?}", first_column);
            }
        }
        Err(e) => eprintln!("Processing failed: {}", e),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let processor = CsvProcessor::new(',', 3);
        let result = processor.process_file(temp_file.path().to_str().unwrap());
        
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
    }

    #[test]
    fn test_invalid_column_count() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30").unwrap();

        let processor = CsvProcessor::new(',', 3);
        let result = processor.process_file(temp_file.path().to_str().unwrap());
        
        assert!(result.is_err());
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

pub fn parse_csv_file<P: AsRef<Path>>(file_path: P) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut line_number = 0;

    for line in reader.lines() {
        line_number += 1;
        let line_content = line?;
        
        if line_content.trim().is_empty() || line_content.starts_with('#') {
            continue;
        }

        let fields: Vec<&str> = line_content.split(',').collect();
        
        if fields.len() != 4 {
            return Err(format!("Invalid field count at line {}", line_number).into());
        }

        let id = fields[0].parse::<u32>()
            .map_err(|e| format!("Invalid ID at line {}: {}", line_number, e))?;
        
        let name = fields[1].trim().to_string();
        if name.is_empty() {
            return Err(format!("Empty name field at line {}", line_number).into());
        }

        let value = fields[2].parse::<f64>()
            .map_err(|e| format!("Invalid value at line {}: {}", line_number, e))?;
        
        let active = match fields[3].trim().to_lowercase().as_str() {
            "true" | "1" | "yes" => true,
            "false" | "0" | "no" => false,
            _ => return Err(format!("Invalid boolean value at line {}", line_number).into()),
        };

        records.push(CsvRecord {
            id,
            name,
            value,
            active,
        });
    }

    if records.is_empty() {
        return Err("No valid records found in CSV file".into());
    }

    Ok(records)
}

pub fn calculate_average_value(records: &[CsvRecord]) -> Option<f64> {
    let active_records: Vec<&CsvRecord> = records.iter()
        .filter(|r| r.active)
        .collect();
    
    if active_records.is_empty() {
        return None;
    }

    let sum: f64 = active_records.iter()
        .map(|r| r.value)
        .sum();
    
    Some(sum / active_records.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_csv_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Alice,42.5,true").unwrap();
        writeln!(temp_file, "2,Bob,37.8,false").unwrap();
        writeln!(temp_file, "3,Charlie,99.9,yes").unwrap();

        let records = parse_csv_file(temp_file.path()).unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].name, "Alice");
        assert_eq!(records[1].active, false);
        assert_eq!(records[2].value, 99.9);
    }

    #[test]
    fn test_average_calculation() {
        let records = vec![
            CsvRecord { id: 1, name: "Test1".to_string(), value: 10.0, active: true },
            CsvRecord { id: 2, name: "Test2".to_string(), value: 20.0, active: true },
            CsvRecord { id: 3, name: "Test3".to_string(), value: 30.0, active: false },
        ];

        let avg = calculate_average_value(&records).unwrap();
        assert_eq!(avg, 15.0);
    }

    #[test]
    fn test_invalid_csv_format() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Alice,42.5").unwrap();

        let result = parse_csv_file(temp_file.path());
        assert!(result.is_err());
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

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

    pub fn filter_rows<P: AsRef<Path>>(
        &self,
        input_path: P,
        output_path: P,
        predicate: impl Fn(&[String]) -> bool,
    ) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(output_path)?;
        let mut lines = reader.lines();
        let mut processed_count = 0;

        if self.has_headers {
            if let Some(header) = lines.next() {
                writeln!(output_file, "{}", header?)?;
            }
        }

        for line in lines {
            let line = line?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if predicate(&fields) {
                writeln!(output_file, "{}", line)?;
                processed_count += 1;
            }
        }

        Ok(processed_count)
    }

    pub fn transform_column<P: AsRef<Path>>(
        &self,
        input_path: P,
        output_path: P,
        column_index: usize,
        transformer: impl Fn(&str) -> String,
    ) -> Result<(), Box<dyn Error>> {
        let input_file = File::open(input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(output_path)?;
        let mut lines = reader.lines();

        if self.has_headers {
            if let Some(header) = lines.next() {
                writeln!(output_file, "{}", header?)?;
            }
        }

        for line in lines {
            let line = line?;
            let mut fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if column_index < fields.len() {
                fields[column_index] = transformer(&fields[column_index]);
            }

            let transformed_line = fields.join(&self.delimiter.to_string());
            writeln!(output_file, "{}", transformed_line)?;
        }

        Ok(())
    }
}

pub fn validate_csv_format(content: &str, delimiter: char) -> bool {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return false;
    }

    let expected_fields = lines[0].split(delimiter).count();
    lines.iter().all(|line| line.split(delimiter).count() == expected_fields)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_filter_rows() {
        let input_content = "name,age,city\nAlice,30,London\nBob,25,Paris\nCharlie,35,Tokyo";
        let mut input_file = NamedTempFile::new().unwrap();
        write!(input_file, "{}", input_content).unwrap();

        let output_file = NamedTempFile::new().unwrap();
        let processor = CsvProcessor::new(',', true);

        let result = processor.filter_rows(
            input_file.path(),
            output_file.path(),
            |fields| fields[1].parse::<i32>().unwrap_or(0) >= 30,
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);

        let mut output_content = String::new();
        File::open(output_file.path())
            .unwrap()
            .read_to_string(&mut output_content)
            .unwrap();

        assert!(output_content.contains("Alice,30,London"));
        assert!(output_content.contains("Charlie,35,Tokyo"));
        assert!(!output_content.contains("Bob,25,Paris"));
    }

    #[test]
    fn test_transform_column() {
        let input_content = "product,price\nApple,1.50\nBanana,0.75";
        let mut input_file = NamedTempFile::new().unwrap();
        write!(input_file, "{}", input_content).unwrap();

        let output_file = NamedTempFile::new().unwrap();
        let processor = CsvProcessor::new(',', true);

        let result = processor.transform_column(
            input_file.path(),
            output_file.path(),
            1,
            |price| format!("${}", price),
        );

        assert!(result.is_ok());

        let mut output_content = String::new();
        File::open(output_file.path())
            .unwrap()
            .read_to_string(&mut output_content)
            .unwrap();

        assert!(output_content.contains("$1.50"));
        assert!(output_content.contains("$0.75"));
    }

    #[test]
    fn test_validate_csv_format() {
        let valid_csv = "a,b,c\n1,2,3\nx,y,z";
        assert!(validate_csv_format(valid_csv, ','));

        let invalid_csv = "a,b,c\n1,2\nx,y,z";
        assert!(!validate_csv_format(invalid_csv, ','));
    }
}