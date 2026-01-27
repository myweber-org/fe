use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

#[derive(Debug)]
pub struct CsvRecord {
    pub columns: Vec<String>,
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

    pub fn parse_file<T: AsRef<str>>(&self, path: T) -> Result<Vec<CsvRecord>, CsvError> {
        let file = File::open(path.as_ref())?;
        let reader = BufReader::new(file);
        self.parse(reader)
    }

    pub fn parse<R: BufRead>(&self, reader: R) -> Result<Vec<CsvRecord>, CsvError> {
        let mut records = Vec::new();
        let mut lines = reader.lines().enumerate();

        if self.has_header {
            lines.next();
        }

        for (line_num, line) in lines {
            let line = line?;
            let columns: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if columns.is_empty() {
                continue;
            }

            records.push(CsvRecord { columns });
        }

        Ok(records)
    }

    pub fn parse_typed<T: FromStr>(record: &CsvRecord, index: usize) -> Result<T, CsvError>
    where
        T::Err: std::fmt::Debug,
    {
        record.columns
            .get(index)
            .ok_or_else(|| CsvError::ParseError(format!("Column index {} out of bounds", index)))
            .and_then(|s| {
                s.parse::<T>()
                    .map_err(|_| CsvError::ParseError(format!("Failed to parse value: {}", s)))
            })
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
    use std::io::Cursor;

    #[test]
    fn test_basic_parsing() {
        let data = "name,age,city\nJohn,30,New York\nJane,25,London";
        let cursor = Cursor::new(data);
        let parser = CsvParser::new();
        let result = parser.parse(cursor).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].columns, vec!["John", "30", "New York"]);
        assert_eq!(result[1].columns, vec!["Jane", "25", "London"]);
    }

    #[test]
    fn test_typed_parsing() {
        let record = CsvRecord {
            columns: vec!["42".to_string(), "3.14".to_string()],
        };

        let int_val: i32 = CsvParser::parse_typed(&record, 0).unwrap();
        let float_val: f64 = CsvParser::parse_typed(&record, 1).unwrap();

        assert_eq!(int_val, 42);
        assert_eq!(float_val, 3.14);
    }

    #[test]
    fn test_custom_delimiter() {
        let data = "name|age|city\nJohn|30|New York";
        let cursor = Cursor::new(data);
        let parser = CsvParser::new().delimiter('|');
        let result = parser.parse(cursor).unwrap();

        assert_eq!(result[0].columns, vec!["John", "30", "New York"]);
    }
}