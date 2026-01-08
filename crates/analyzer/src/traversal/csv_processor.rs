use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvRecord {
    pub fields: Vec<String>,
}

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
    expected_columns: usize,
}

impl CsvProcessor {
    pub fn new(delimiter: char, expected_columns: usize) -> Self {
        CsvProcessor {
            delimiter,
            expected_columns,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<CsvRecord>, CsvError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let record = self.parse_line(&line, line_num + 1)?;
            records.push(record);
        }

        Ok(records)
    }

    fn parse_line(&self, line: &str, line_num: usize) -> Result<CsvRecord, CsvError> {
        let fields: Vec<String> = line
            .split(self.delimiter)
            .map(|s| s.trim().to_string())
            .collect();

        if fields.len() != self.expected_columns {
            return Err(CsvError::ValidationError(format!(
                "Line {}: expected {} columns, found {}",
                line_num,
                self.expected_columns,
                fields.len()
            )));
        }

        for (idx, field) in fields.iter().enumerate() {
            if field.is_empty() {
                return Err(CsvError::ParseError(format!(
                    "Line {}: empty field at column {}",
                    line_num,
                    idx + 1
                )));
            }
        }

        Ok(CsvRecord { fields })
    }

    pub fn extract_column(&self, records: &[CsvRecord], column_index: usize) -> Vec<String> {
        records
            .iter()
            .filter_map(|record| record.fields.get(column_index).cloned())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "John,Doe,30").unwrap();
        writeln!(temp_file, "Jane,Smith,25").unwrap();

        let processor = CsvProcessor::new(',', 3);
        let result = processor.process_file(temp_file.path());

        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].fields, vec!["John", "Doe", "30"]);
    }

    #[test]
    fn test_invalid_column_count() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "John,Doe,30,Extra").unwrap();

        let processor = CsvProcessor::new(',', 3);
        let result = processor.process_file(temp_file.path());

        assert!(matches!(result, Err(CsvError::ValidationError(_))));
    }

    #[test]
    fn test_column_extraction() {
        let records = vec![
            CsvRecord {
                fields: vec!["A".to_string(), "B".to_string(), "C".to_string()],
            },
            CsvRecord {
                fields: vec!["D".to_string(), "E".to_string(), "F".to_string()],
            },
        ];

        let processor = CsvProcessor::new(',', 3);
        let column = processor.extract_column(&records, 1);
        assert_eq!(column, vec!["B", "E"]);
    }
}