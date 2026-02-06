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
}

impl CsvParser {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        CsvParser {
            delimiter,
            has_header,
        }
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
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

    pub fn validate_records(&self, records: &[CsvRecord], expected_columns: usize) -> Result<(), String> {
        for (idx, record) in records.iter().enumerate() {
            if record.columns.len() != expected_columns {
                return Err(format!(
                    "Record {} has {} columns, expected {}",
                    idx + 1,
                    record.columns.len(),
                    expected_columns
                ));
            }
        }
        Ok(())
    }
}

pub fn process_csv_data(records: &[CsvRecord]) -> Vec<Vec<String>> {
    records
        .iter()
        .map(|record| record.columns.clone())
        .collect()
}