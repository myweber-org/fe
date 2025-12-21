use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

#[derive(Debug)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub active: bool,
}

#[derive(Debug)]
pub enum CsvError {
    IoError(std::io::Error),
    ParseError(String),
    InvalidFormat(String),
}

impl From<std::io::Error> for CsvError {
    fn from(err: std::io::Error) -> Self {
        CsvError::IoError(err)
    }
}

pub struct CsvParser {
    delimiter: char,
    has_header: bool,
}

impl CsvParser {
    pub fn new() -> Self {
        CsvParser {
            delimiter: ',',
            has_header: true,
        }
    }

    pub fn with_delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = delimiter;
        self
    }

    pub fn with_header(mut self, has_header: bool) -> Self {
        self.has_header = has_header;
        self
    }

    pub fn parse_file(&self, path: &str) -> Result<Vec<CsvRecord>, CsvError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 && self.has_header {
                continue;
            }

            if line.trim().is_empty() {
                continue;
            }

            let record = self.parse_line(&line, line_num + 1)?;
            records.push(record);
        }

        Ok(records)
    }

    fn parse_line(&self, line: &str, line_num: usize) -> Result<CsvRecord, CsvError> {
        let parts: Vec<&str> = line.split(self.delimiter).collect();
        
        if parts.len() != 4 {
            return Err(CsvError::InvalidFormat(
                format!("Line {}: expected 4 fields, found {}", line_num, parts.len())
            ));
        }

        let id = u32::from_str(parts[0]).map_err(|e| {
            CsvError::ParseError(format!("Line {}: invalid id '{}': {}", line_num, parts[0], e))
        })?;

        let name = parts[1].trim().to_string();
        if name.is_empty() {
            return Err(CsvError::InvalidFormat(
                format!("Line {}: name cannot be empty", line_num)
            ));
        }

        let value = f64::from_str(parts[2]).map_err(|e| {
            CsvError::ParseError(format!("Line {}: invalid value '{}': {}", line_num, parts[2], e))
        })?;

        let active = bool::from_str(parts[3]).map_err(|e| {
            CsvError::ParseError(format!("Line {}: invalid active flag '{}': {}", line_num, parts[3], e))
        })?;

        Ok(CsvRecord {
            id,
            name,
            value,
            active,
        })
    }

    pub fn calculate_stats(records: &[CsvRecord]) -> (f64, f64, usize) {
        if records.is_empty() {
            return (0.0, 0.0, 0);
        }

        let sum: f64 = records.iter().map(|r| r.value).sum();
        let avg = sum / records.len() as f64;
        let active_count = records.iter().filter(|r| r.active).count();

        (sum, avg, active_count)
    }
}

impl Default for CsvParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,active").unwrap();
        writeln!(temp_file, "1,Alice,42.5,true").unwrap();
        writeln!(temp_file, "2,Bob,37.8,false").unwrap();
        writeln!(temp_file, "3,Charlie,99.9,true").unwrap();

        let parser = CsvParser::new();
        let records = parser.parse_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(records.len(), 3);
        assert_eq!(records[0].name, "Alice");
        assert_eq!(records[1].value, 37.8);
        assert!(records[2].active);

        let (sum, avg, active_count) = CsvParser::calculate_stats(&records);
        assert!((sum - 180.2).abs() < 0.001);
        assert!((avg - 60.066).abs() < 0.001);
        assert_eq!(active_count, 2);
    }

    #[test]
    fn test_invalid_format() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Alice,42.5").unwrap();

        let parser = CsvParser::new().with_header(false);
        let result = parser.parse_file(temp_file.path().to_str().unwrap());
        
        assert!(matches!(result, Err(CsvError::InvalidFormat(_))));
    }
}