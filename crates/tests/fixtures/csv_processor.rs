use std::error::Error;
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

pub struct CsvProcessor {
    records: Vec<CsvRecord>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), CsvError> {
        let file = File::open(&path).map_err(|e| CsvError::IoError(e.to_string()))?;
        let reader = BufReader::new(file);

        for (line_num, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| CsvError::IoError(e.to_string()))?;
            
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                return Err(CsvError::ParseError(
                    format!("Line {}: Expected 4 columns, found {}", line_num + 1, parts.len())
                ));
            }

            let id = parts[0].parse::<u32>()
                .map_err(|e| CsvError::ParseError(
                    format!("Line {}: Invalid ID '{}': {}", line_num + 1, parts[0], e)
                ))?;

            let name = parts[1].trim().to_string();
            if name.is_empty() {
                return Err(CsvError::ValidationError(
                    format!("Line {}: Name cannot be empty", line_num + 1)
                ));
            }

            let value = parts[2].parse::<f64>()
                .map_err(|e| CsvError::ParseError(
                    format!("Line {}: Invalid value '{}': {}", line_num + 1, parts[2], e)
                ))?;

            let active = match parts[3].trim().to_lowercase().as_str() {
                "true" | "1" => true,
                "false" | "0" => false,
                _ => return Err(CsvError::ParseError(
                    format!("Line {}: Invalid boolean '{}'", line_num + 1, parts[3])
                )),
            };

            self.records.push(CsvRecord {
                id,
                name,
                value,
                active,
            });
        }

        Ok(())
    }

    pub fn get_active_records(&self) -> Vec<&CsvRecord> {
        self.records.iter().filter(|r| r.active).collect()
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }

    pub fn find_by_name(&self, name: &str) -> Option<&CsvRecord> {
        self.records.iter().find(|r| r.name == name)
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Alice,42.5,true").unwrap();
        writeln!(temp_file, "2,Bob,33.7,false").unwrap();
        writeln!(temp_file, "3,Charlie,19.2,true").unwrap();

        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(processor.record_count(), 3);
        assert_eq!(processor.get_active_records().len(), 2);
        assert!((processor.calculate_total_value() - 95.4).abs() < 0.001);
        assert!(processor.find_by_name("Bob").is_some());
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvRecord {
    pub columns: Vec<String>,
}

pub struct CsvProcessor {
    delimiter: char,
    has_header: bool,
}

impl CsvProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        CsvProcessor {
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

        for (line_num, line_result) in lines {
            let line = line_result?;
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

    pub fn filter_records<F>(&self, records: Vec<CsvRecord>, predicate: F) -> Vec<CsvRecord>
    where
        F: Fn(&CsvRecord) -> bool,
    {
        records.into_iter().filter(predicate).collect()
    }

    pub fn extract_column(&self, records: &[CsvRecord], column_index: usize) -> Vec<String> {
        records
            .iter()
            .filter_map(|record| record.columns.get(column_index).cloned())
            .collect()
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
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let processor = CsvProcessor::new(',', true);
        let records = processor.parse_file(temp_file.path()).unwrap();

        assert_eq!(records.len(), 2);
        assert_eq!(records[0].columns, vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_filter_records() {
        let records = vec![
            CsvRecord {
                columns: vec!["A".to_string(), "10".to_string()],
            },
            CsvRecord {
                columns: vec!["B".to_string(), "20".to_string()],
            },
        ];

        let processor = CsvProcessor::new(',', false);
        let filtered = processor.filter_records(records, |r| r.columns[0] == "A");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].columns[0], "A");
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvConfig {
    delimiter: char,
    selected_columns: Vec<usize>,
    skip_header: bool,
}

impl Default for CsvConfig {
    fn default() -> Self {
        CsvConfig {
            delimiter: ',',
            selected_columns: Vec::new(),
            skip_header: false,
        }
    }
}

pub struct CsvProcessor {
    config: CsvConfig,
}

impl CsvProcessor {
    pub fn new(config: CsvConfig) -> Self {
        CsvProcessor { config }
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut results = Vec::new();
        let mut line_number = 0;

        for line in reader.lines() {
            let line = line?;
            line_number += 1;

            if self.config.skip_header && line_number == 1 {
                continue;
            }

            let parsed_row = self.parse_line(&line);
            results.push(parsed_row);
        }

        Ok(results)
    }

    fn parse_line(&self, line: &str) -> Vec<String> {
        let parts: Vec<&str> = line.split(self.config.delimiter).collect();
        
        if self.config.selected_columns.is_empty() {
            parts.iter().map(|&s| s.to_string()).collect()
        } else {
            self.config.selected_columns
                .iter()
                .filter_map(|&idx| parts.get(idx).map(|&s| s.to_string()))
                .collect()
        }
    }

    pub fn filter_rows<F>(&self, rows: Vec<Vec<String>>, predicate: F) -> Vec<Vec<String>>
    where
        F: Fn(&[String]) -> bool,
    {
        rows.into_iter()
            .filter(|row| predicate(row))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_parsing() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "name,age,city")?;
        writeln!(temp_file, "Alice,30,London")?;
        writeln!(temp_file, "Bob,25,Paris")?;

        let config = CsvConfig {
            delimiter: ',',
            selected_columns: vec![0, 2],
            skip_header: true,
        };

        let processor = CsvProcessor::new(config);
        let result = processor.process_file(temp_file.path())?;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "London"]);
        assert_eq!(result[1], vec!["Bob", "Paris"]);

        Ok(())
    }

    #[test]
    fn test_row_filtering() {
        let rows = vec![
            vec!["apple".to_string(), "10".to_string()],
            vec!["banana".to_string(), "5".to_string()],
            vec!["orange".to_string(), "15".to_string()],
        ];

        let config = CsvConfig::default();
        let processor = CsvProcessor::new(config);
        
        let filtered = processor.filter_rows(rows, |row| {
            row.get(1)
                .and_then(|qty| qty.parse::<i32>().ok())
                .map_or(false, |qty| qty > 5)
        });

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().any(|row| row[0] == "apple"));
        assert!(filtered.iter().any(|row| row[0] == "orange"));
    }
}