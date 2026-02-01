use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct CsvRecord {
    pub columns: Vec<String>,
}

#[derive(Debug)]
pub struct CsvParser {
    delimiter: char,
    has_header: bool,
    header: Option<Vec<String>>,
}

impl CsvParser {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        CsvParser {
            delimiter,
            has_header,
            header: None,
        }
    }

    pub fn parse_file<P: AsRef<Path>>(&mut self, path: P) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines().enumerate();

        if self.has_header {
            if let Some((_, first_line)) = lines.next() {
                let header_line = first_line?;
                self.header = Some(self.parse_line(&header_line)?);
            }
        }

        for (line_num, line_result) in lines {
            let line = line_result?;
            let columns = self.parse_line(&line)?;
            
            if !columns.is_empty() {
                records.push(CsvRecord { columns });
            } else {
                eprintln!("Warning: Empty line at position {}", line_num + 1);
            }
        }

        Ok(records)
    }

    fn parse_line(&self, line: &str) -> Result<Vec<String>, Box<dyn Error>> {
        let mut columns = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut chars = line.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '"' => {
                    if in_quotes && chars.peek() == Some(&'"') {
                        current.push('"');
                        chars.next();
                    } else {
                        in_quotes = !in_quotes;
                    }
                }
                _ if ch == self.delimiter && !in_quotes => {
                    columns.push(current.trim().to_string());
                    current.clear();
                }
                _ => {
                    current.push(ch);
                }
            }
        }

        columns.push(current.trim().to_string());
        Ok(columns)
    }

    pub fn get_header(&self) -> Option<&Vec<String>> {
        self.header.as_ref()
    }

    pub fn validate_records(&self, records: &[CsvRecord]) -> Result<(), String> {
        if let Some(header) = &self.header {
            let expected_len = header.len();
            for (i, record) in records.iter().enumerate() {
                if record.columns.len() != expected_len {
                    return Err(format!(
                        "Record {} has {} columns, expected {}",
                        i + 1,
                        record.columns.len(),
                        expected_len
                    ));
                }
            }
        }
        Ok(())
    }
}

pub fn count_records(records: &[CsvRecord]) -> usize {
    records.len()
}

pub fn get_column_data(records: &[CsvRecord], column_index: usize) -> Vec<&str> {
    records
        .iter()
        .filter_map(|record| record.columns.get(column_index))
        .map(|s| s.as_str())
        .collect()
}