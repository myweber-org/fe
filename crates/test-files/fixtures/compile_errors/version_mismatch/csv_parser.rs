use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct CsvRecord {
    pub fields: Vec<String>,
}

#[derive(Debug)]
pub enum CsvError {
    IoError(std::io::Error),
    ParseError(String),
}

impl From<std::io::Error> for CsvError {
    fn from(err: std::io::Error) -> Self {
        CsvError::IoError(err)
    }
}

pub struct CsvParser {
    reader: BufReader<File>,
    delimiter: char,
}

impl CsvParser {
    pub fn new(file_path: &str, delimiter: char) -> Result<Self, CsvError> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        Ok(CsvParser { reader, delimiter })
    }

    pub fn parse_next(&mut self) -> Result<Option<CsvRecord>, CsvError> {
        let mut line = String::new();
        let bytes_read = self.reader.read_line(&mut line)?;

        if bytes_read == 0 {
            return Ok(None);
        }

        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            return Ok(None);
        }

        let fields: Vec<String> = trimmed
            .split(self.delimiter)
            .map(|s| s.trim().to_string())
            .collect();

        if fields.is_empty() {
            return Err(CsvError::ParseError("Empty record found".to_string()));
        }

        Ok(Some(CsvRecord { fields }))
    }

    pub fn parse_all(&mut self) -> Result<Vec<CsvRecord>, CsvError> {
        let mut records = Vec::new();
        while let Some(record) = self.parse_next()? {
            records.push(record);
        }
        Ok(records)
    }
}

pub fn validate_csv_records(records: &[CsvRecord]) -> Result<(), CsvError> {
    if records.is_empty() {
        return Err(CsvError::ParseError("No records found".to_string()));
    }

    let expected_field_count = records[0].fields.len();
    for (index, record) in records.iter().enumerate() {
        if record.fields.len() != expected_field_count {
            return Err(CsvError::ParseError(format!(
                "Record {} has {} fields, expected {}",
                index + 1,
                record.fields.len(),
                expected_field_count
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let file_path = temp_file.path().to_str().unwrap();
        let mut parser = CsvParser::new(file_path, ',').unwrap();
        let records = parser.parse_all().unwrap();

        assert_eq!(records.len(), 2);
        assert_eq!(records[0].fields, vec!["Alice", "30", "New York"]);
        assert_eq!(records[1].fields, vec!["Bob", "25", "London"]);
    }

    #[test]
    fn test_validation_success() {
        let records = vec![
            CsvRecord {
                fields: vec!["a".to_string(), "b".to_string()],
            },
            CsvRecord {
                fields: vec!["c".to_string(), "d".to_string()],
            },
        ];
        assert!(validate_csv_records(&records).is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let records = vec![
            CsvRecord {
                fields: vec!["a".to_string(), "b".to_string()],
            },
            CsvRecord {
                fields: vec!["c".to_string()],
            },
        ];
        assert!(validate_csv_records(&records).is_err());
    }
}