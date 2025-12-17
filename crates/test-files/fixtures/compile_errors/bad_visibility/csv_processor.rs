use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

#[derive(Debug)]
pub enum ParseError {
    IoError(std::io::Error),
    InvalidFormat(String),
    InvalidNumber(String),
    InvalidBoolean(String),
    MissingField(usize),
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::IoError(err)
    }
}

pub fn parse_csv<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, ParseError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }

        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() < 4 {
            return Err(ParseError::MissingField(line_num + 1));
        }

        let id = fields[0]
            .trim()
            .parse::<u32>()
            .map_err(|_| ParseError::InvalidNumber(fields[0].to_string()))?;

        let name = fields[1].trim().to_string();
        if name.is_empty() {
            return Err(ParseError::InvalidFormat("Name cannot be empty".to_string()));
        }

        let value = fields[2]
            .trim()
            .parse::<f64>()
            .map_err(|_| ParseError::InvalidNumber(fields[2].to_string()))?;

        let active = match fields[3].trim().to_lowercase().as_str() {
            "true" | "1" | "yes" => true,
            "false" | "0" | "no" => false,
            _ => return Err(ParseError::InvalidBoolean(fields[3].to_string())),
        };

        records.push(Record {
            id,
            name,
            value,
            active,
        });
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, usize) {
    if records.is_empty() {
        return (0.0, 0.0, 0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let mean = sum / records.len() as f64;
    let active_count = records.iter().filter(|r| r.active).count();

    (sum, mean, active_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Alice,42.5,true").unwrap();
        writeln!(temp_file, "2,Bob,37.2,false").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "3,Charlie,19.8,yes").unwrap();

        let records = parse_csv(temp_file.path()).unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].name, "Alice");
        assert_eq!(records[1].active, false);
        assert_eq!(records[2].id, 3);
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            Record {
                id: 1,
                name: "Test1".to_string(),
                value: 10.0,
                active: true,
            },
            Record {
                id: 2,
                name: "Test2".to_string(),
                value: 20.0,
                active: false,
            },
            Record {
                id: 3,
                name: "Test3".to_string(),
                value: 30.0,
                active: true,
            },
        ];

        let (sum, mean, active_count) = calculate_statistics(&records);
        assert_eq!(sum, 60.0);
        assert_eq!(mean, 20.0);
        assert_eq!(active_count, 2);
    }

    #[test]
    fn test_parse_error_handling() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "invalid,Data,42.5,true").unwrap();

        let result = parse_csv(temp_file.path());
        assert!(matches!(result, Err(ParseError::InvalidNumber(_))));
    }
}