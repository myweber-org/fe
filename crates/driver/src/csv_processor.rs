use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

pub struct CsvFilter {
    pub column_index: usize,
    pub filter_value: String,
}

impl CsvFilter {
    pub fn new(column_index: usize, filter_value: &str) -> Self {
        CsvFilter {
            column_index,
            filter_value: filter_value.to_string(),
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut filtered_rows = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let columns: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            
            if columns.len() > self.column_index && columns[self.column_index] == self.filter_value {
                filtered_rows.push(columns);
            }
        }

        Ok(filtered_rows)
    }

    pub fn count_matches<P: AsRef<Path>>(&self, file_path: P) -> Result<usize, Box<dyn Error>> {
        let matches = self.process_file(file_path)?;
        Ok(matches.len())
    }
}

pub fn read_csv_headers<P: AsRef<Path>>(file_path: P) -> Result<Vec<String>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);
    let mut header_line = String::new();
    reader.read_line(&mut header_line)?;
    
    let headers: Vec<String> = header_line.trim().split(',').map(|s| s.trim().to_string()).collect();
    Ok(headers)
}

pub fn write_filtered_csv<P: AsRef<Path>>(
    filter: &CsvFilter,
    input_path: P,
    output_path: P,
) -> Result<(), Box<dyn Error>> {
    use std::io::Write;
    
    let filtered_data = filter.process_file(input_path)?;
    let mut output_file = File::create(output_path)?;
    
    for row in filtered_data {
        let line = row.join(",");
        writeln!(output_file, "{}", line)?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_filter() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        writeln!(temp_file, "Charlie,30,Paris").unwrap();
        
        let filter = CsvFilter::new(1, "30");
        let result = filter.process_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0][0], "Alice");
        assert_eq!(result[1][0], "Charlie");
    }

    #[test]
    fn test_count_matches() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,status").unwrap();
        writeln!(temp_file, "1,active").unwrap();
        writeln!(temp_file, "2,inactive").unwrap();
        writeln!(temp_file, "3,active").unwrap();
        
        let filter = CsvFilter::new(1, "active");
        let count = filter.count_matches(temp_file.path()).unwrap();
        
        assert_eq!(count, 2);
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
}

#[derive(Debug)]
pub enum CsvError {
    IoError(String),
    ParseError(String),
    ValidationError(String),
}

impl std::fmt::Display for CsvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CsvError::IoError(msg) => write!(f, "IO Error: {}", msg),
            CsvError::ParseError(msg) => write!(f, "Parse Error: {}", msg),
            CsvError::ValidationError(msg) => write!(f, "Validation Error: {}", msg),
        }
    }
}

impl Error for CsvError {}

pub fn process_csv_file<P: AsRef<Path>>(file_path: P) -> Result<Vec<CsvRecord>, CsvError> {
    let file = File::open(&file_path).map_err(|e| {
        CsvError::IoError(format!("Failed to open file {}: {}", file_path.as_ref().display(), e))
    })?;

    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut line_number = 0;

    for line in reader.lines() {
        line_number += 1;
        let line_content = line.map_err(|e| {
            CsvError::IoError(format!("Failed to read line {}: {}", line_number, e))
        })?;

        if line_content.trim().is_empty() || line_content.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line_content.split(',').collect();
        if parts.len() != 3 {
            return Err(CsvError::ParseError(format!(
                "Line {}: Expected 3 columns, found {}",
                line_number,
                parts.len()
            )));
        }

        let id = parts[0].parse::<u32>().map_err(|_| {
            CsvError::ParseError(format!("Line {}: Invalid ID format '{}'", line_number, parts[0]))
        })?;

        let name = parts[1].trim().to_string();
        if name.is_empty() {
            return Err(CsvError::ValidationError(format!(
                "Line {}: Name cannot be empty",
                line_number
            )));
        }

        let value = parts[2].parse::<f64>().map_err(|_| {
            CsvError::ParseError(format!(
                "Line {}: Invalid value format '{}'",
                line_number, parts[2]
            ))
        })?;

        if value < 0.0 {
            return Err(CsvError::ValidationError(format!(
                "Line {}: Value cannot be negative: {}",
                line_number, value
            )));
        }

        records.push(CsvRecord { id, name, value });
    }

    if records.is_empty() {
        return Err(CsvError::ValidationError(
            "CSV file contains no valid records".to_string(),
        ));
    }

    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Alice,42.5").unwrap();
        writeln!(temp_file, "2,Bob,17.8").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "3,Charlie,99.9").unwrap();

        let result = process_csv_file(temp_file.path());
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].name, "Alice");
        assert_eq!(records[1].value, 17.8);
        assert_eq!(records[2].id, 3);
    }

    #[test]
    fn test_invalid_column_count() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Alice").unwrap();

        let result = process_csv_file(temp_file.path());
        assert!(matches!(result, Err(CsvError::ParseError(_))));
    }

    #[test]
    fn test_negative_value() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Alice,-5.0").unwrap();

        let result = process_csv_file(temp_file.path());
        assert!(matches!(result, Err(CsvError::ValidationError(_))));
    }

    #[test]
    fn test_empty_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let result = process_csv_file(temp_file.path());
        assert!(matches!(result, Err(CsvError::ValidationError(_))));
    }
}