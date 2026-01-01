use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    file_path: String,
    delimiter: char,
}

impl CsvProcessor {
    pub fn new(file_path: &str, delimiter: char) -> Self {
        CsvProcessor {
            file_path: file_path.to_string(),
            delimiter,
        }
    }

    pub fn filter_rows<F>(&self, predicate: F) -> Result<Vec<Vec<String>>, Box<dyn Error>>
    where
        F: Fn(&[String]) -> bool,
    {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut filtered_data = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if predicate(&fields) {
                filtered_data.push(fields);
            }
        }

        Ok(filtered_data)
    }

    pub fn count_rows(&self) -> Result<usize, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        Ok(reader.lines().count())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_filter_rows() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,London").unwrap();
        writeln!(temp_file, "Bob,25,Paris").unwrap();
        writeln!(temp_file, "Charlie,35,Tokyo").unwrap();

        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap(), ',');
        let result = processor
            .filter_rows(|fields| fields.get(1).and_then(|a| a.parse::<i32>().ok()).unwrap_or(0) > 30)
            .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0][0], "Charlie");
    }

    #[test]
    fn test_count_rows() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "header1,header2").unwrap();
        writeln!(temp_file, "value1,value2").unwrap();
        writeln!(temp_file, "value3,value4").unwrap();

        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap(), ',');
        assert_eq!(processor.count_rows().unwrap(), 3);
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

        let record = parse_csv_line(&line_content, line_number)?;
        records.push(record);
    }

    if records.is_empty() {
        return Err(CsvError::ValidationError(
            "CSV file contains no valid records".to_string(),
        ));
    }

    Ok(records)
}

fn parse_csv_line(line: &str, line_number: usize) -> Result<CsvRecord, CsvError> {
    let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();

    if parts.len() != 4 {
        return Err(CsvError::ParseError(format!(
            "Line {}: Expected 4 fields, found {}",
            line_number,
            parts.len()
        )));
    }

    let id = parts[0]
        .parse::<u32>()
        .map_err(|e| CsvError::ParseError(format!("Line {}: Invalid ID '{}': {}", line_number, parts[0], e)))?;

    let name = parts[1].to_string();
    if name.is_empty() {
        return Err(CsvError::ValidationError(format!(
            "Line {}: Name cannot be empty",
            line_number
        )));
    }

    let value = parts[2]
        .parse::<f64>()
        .map_err(|e| CsvError::ParseError(format!("Line {}: Invalid value '{}': {}", line_number, parts[2], e)))?;

    if value < 0.0 {
        return Err(CsvError::ValidationError(format!(
            "Line {}: Value cannot be negative: {}",
            line_number, value
        )));
    }

    let active = parts[3]
        .parse::<bool>()
        .map_err(|e| CsvError::ParseError(format!("Line {}: Invalid boolean '{}': {}", line_number, parts[3], e)))?;

    Ok(CsvRecord {
        id,
        name,
        value,
        active,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Alice,42.5,true").unwrap();
        writeln!(temp_file, "2,Bob,100.0,false").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "3,Charlie,0.0,true").unwrap();

        let result = process_csv_file(temp_file.path());
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].name, "Alice");
        assert_eq!(records[1].value, 100.0);
        assert_eq!(records[2].active, true);
    }

    #[test]
    fn test_invalid_csv_format() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Alice,42.5").unwrap();

        let result = process_csv_file(temp_file.path());
        assert!(result.is_err());
        if let Err(CsvError::ParseError(msg)) = result {
            assert!(msg.contains("Expected 4 fields"));
        } else {
            panic!("Expected ParseError");
        }
    }

    #[test]
    fn test_negative_value_validation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Alice,-10.0,true").unwrap();

        let result = process_csv_file(temp_file.path());
        assert!(result.is_err());
        if let Err(CsvError::ValidationError(msg)) = result {
            assert!(msg.contains("Value cannot be negative"));
        } else {
            panic!("Expected ValidationError");
        }
    }
}