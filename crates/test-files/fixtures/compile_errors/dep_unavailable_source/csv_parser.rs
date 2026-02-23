use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
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

    pub fn delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = delimiter;
        self
    }

    pub fn has_header(mut self, has_header: bool) -> Self {
        self.has_header = has_header;
        self
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (line_number, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_number == 0 && self.has_header {
                continue;
            }

            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|field| field.trim().to_string())
                .collect();

            if !record.is_empty() {
                records.push(record);
            }
        }

        Ok(records)
    }

    pub fn parse_string(&self, content: &str) -> Vec<Vec<String>> {
        let mut records = Vec::new();
        
        for line in content.lines() {
            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|field| field.trim().to_string())
                .collect();

            if !record.is_empty() {
                records.push(record);
            }
        }

        records
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parsing() {
        let parser = CsvParser::new();
        let content = "name,age,city\nJohn,30,New York\nJane,25,London";
        let records = parser.parse_string(content);
        
        assert_eq!(records.len(), 2);
        assert_eq!(records[0], vec!["John", "30", "New York"]);
        assert_eq!(records[1], vec!["Jane", "25", "London"]);
    }

    #[test]
    fn test_custom_delimiter() {
        let parser = CsvParser::new().delimiter(';');
        let content = "name;age;city\nJohn;30;New York";
        let records = parser.parse_string(content);
        
        assert_eq!(records[0], vec!["John", "30", "New York"]);
    }

    #[test]
    fn test_no_header() {
        let parser = CsvParser::new().has_header(false);
        let content = "John,30,New York\nJane,25,London";
        let records = parser.parse_string(content);
        
        assert_eq!(records.len(), 2);
        assert_eq!(records[0], vec!["John", "30", "New York"]);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

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
    delimiter: char,
    has_header: bool,
}

impl CsvParser {
    pub fn new() -> Self {
        CsvParser {
            delimiter: ',',
            has_header: false,
        }
    }

    pub fn delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = delimiter;
        self
    }

    pub fn has_header(mut self, has_header: bool) -> Self {
        self.has_header = has_header;
        self
    }

    pub fn parse_file(&self, path: &str) -> Result<Vec<CsvRecord>, CsvError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        self.parse(reader)
    }

    pub fn parse<R: BufRead>(&self, reader: R) -> Result<Vec<CsvRecord>, CsvError> {
        let mut records = Vec::new();
        let mut lines = reader.lines();

        if self.has_header {
            lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let record = self.parse_line(&line)?;
            records.push(record);
        }

        Ok(records)
    }

    fn parse_line(&self, line: &str) -> Result<CsvRecord, CsvError> {
        let fields: Vec<String> = line
            .split(self.delimiter)
            .map(|s| s.trim().to_string())
            .collect();

        if fields.is_empty() {
            return Err(CsvError::ParseError("Empty line".to_string()));
        }

        Ok(CsvRecord { fields })
    }

    pub fn parse_column<T: FromStr>(record: &CsvRecord, index: usize) -> Result<T, CsvError>
    where
        T::Err: std::fmt::Display,
    {
        record
            .fields
            .get(index)
            .ok_or_else(|| CsvError::ParseError(format!("Column index {} out of bounds", index)))?
            .parse()
            .map_err(|e| CsvError::ParseError(format!("Failed to parse column {}: {}", index, e)))
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

    #[test]
    fn test_basic_parsing() {
        let data = "name,age,city\nJohn,30,New York\nJane,25,London";
        let parser = CsvParser::new().has_header(true);
        let result = parser.parse(data.as_bytes()).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].fields, vec!["John", "30", "New York"]);
        assert_eq!(result[1].fields, vec!["Jane", "25", "London"]);
    }

    #[test]
    fn test_column_parsing() {
        let record = CsvRecord {
            fields: vec!["42".to_string(), "3.14".to_string(), "hello".to_string()],
        };

        let int_val: i32 = CsvParser::parse_column(&record, 0).unwrap();
        let float_val: f64 = CsvParser::parse_column(&record, 1).unwrap();
        let string_val: String = CsvParser::parse_column(&record, 2).unwrap();

        assert_eq!(int_val, 42);
        assert_eq!(float_val, 3.14);
        assert_eq!(string_val, "hello");
    }
}