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
    ParseError(String, usize),
    ValidationError(String),
}

impl std::fmt::Display for CsvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CsvError::IoError(msg) => write!(f, "IO error: {}", msg),
            CsvError::ParseError(msg, line) => write!(f, "Parse error at line {}: {}", line, msg),
            CsvError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for CsvError {}

pub struct CsvProcessor {
    delimiter: char,
    has_header: bool,
}

impl Default for CsvProcessor {
    fn default() -> Self {
        Self {
            delimiter: ',',
            has_header: true,
        }
    }
}

impl CsvProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        Self {
            delimiter,
            has_header,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<CsvRecord>, CsvError> {
        let file = File::open(&path).map_err(|e| CsvError::IoError(e.to_string()))?;
        let reader = BufReader::new(file);
        
        let mut records = Vec::new();
        let mut line_number = 0;
        let mut skip_header = self.has_header;

        for line in reader.lines() {
            line_number += 1;
            let line_content = line.map_err(|e| CsvError::IoError(e.to_string()))?;
            
            if skip_header {
                skip_header = false;
                continue;
            }

            if line_content.trim().is_empty() {
                continue;
            }

            let record = self.parse_line(&line_content, line_number)?;
            records.push(record);
        }

        self.validate_records(&records)?;
        Ok(records)
    }

    fn parse_line(&self, line: &str, line_number: usize) -> Result<CsvRecord, CsvError> {
        let parts: Vec<&str> = line.split(self.delimiter).collect();
        
        if parts.len() != 4 {
            return Err(CsvError::ParseError(
                format!("Expected 4 columns, found {}", parts.len()),
                line_number,
            ));
        }

        let id = parts[0].parse::<u32>()
            .map_err(|_| CsvError::ParseError(
                format!("Invalid ID: {}", parts[0]),
                line_number,
            ))?;

        let name = parts[1].trim().to_string();
        if name.is_empty() {
            return Err(CsvError::ParseError(
                "Name cannot be empty".to_string(),
                line_number,
            ));
        }

        let value = parts[2].parse::<f64>()
            .map_err(|_| CsvError::ParseError(
                format!("Invalid value: {}", parts[2]),
                line_number,
            ))?;

        let active = match parts[3].trim().to_lowercase().as_str() {
            "true" | "1" | "yes" => true,
            "false" | "0" | "no" => false,
            _ => return Err(CsvError::ParseError(
                format!("Invalid boolean value: {}", parts[3]),
                line_number,
            )),
        };

        Ok(CsvRecord {
            id,
            name,
            value,
            active,
        })
    }

    fn validate_records(&self, records: &[CsvRecord]) -> Result<(), CsvError> {
        let mut seen_ids = std::collections::HashSet::new();
        
        for record in records {
            if !seen_ids.insert(record.id) {
                return Err(CsvError::ValidationError(
                    format!("Duplicate ID found: {}", record.id)
                ));
            }
            
            if record.value < 0.0 {
                return Err(CsvError::ValidationError(
                    format!("Negative value found for ID {}: {}", record.id, record.value)
                ));
            }
        }
        
        Ok(())
    }

    pub fn calculate_statistics(records: &[CsvRecord]) -> (f64, f64, f64) {
        if records.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        
        let sum: f64 = records.iter().map(|r| r.value).sum();
        let count = records.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = records.iter()
            .map(|r| (r.value - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
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
        writeln!(temp_file, "1,Test Item,42.5,true").unwrap();
        writeln!(temp_file, "2,Another Item,100.0,false").unwrap();
        
        let processor = CsvProcessor::default();
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "Test Item");
        assert_eq!(records[1].value, 100.0);
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            CsvRecord { id: 1, name: "A".to_string(), value: 10.0, active: true },
            CsvRecord { id: 2, name: "B".to_string(), value: 20.0, active: false },
            CsvRecord { id: 3, name: "C".to_string(), value: 30.0, active: true },
        ];
        
        let (mean, variance, std_dev) = CsvProcessor::calculate_statistics(&records);
        
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvConfig {
    pub delimiter: char,
    pub has_headers: bool,
}

impl Default for CsvConfig {
    fn default() -> Self {
        CsvConfig {
            delimiter: ',',
            has_headers: true,
        }
    }
}

pub fn parse_csv<P: AsRef<Path>>(
    path: P,
    config: &CsvConfig,
) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut lines = reader.lines().enumerate();

    if config.has_headers {
        if let Some((_, header_line)) = lines.next() {
            let headers = parse_line(&header_line?, config.delimiter);
            println!("Headers: {:?}", headers);
        }
    }

    for (line_num, line_result) in lines {
        let line = line_result?;
        let fields = parse_line(&line, config.delimiter);
        
        if fields.is_empty() {
            return Err(format!("Line {} is empty", line_num + 1).into());
        }
        
        records.push(fields);
    }

    if records.is_empty() {
        return Err("No data records found".into());
    }

    Ok(records)
}

fn parse_line(line: &str, delimiter: char) -> Vec<String> {
    line.split(delimiter)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn validate_records(records: &[Vec<String>], expected_columns: usize) -> Result<(), String> {
    for (i, record) in records.iter().enumerate() {
        if record.len() != expected_columns {
            return Err(format!(
                "Record {} has {} columns, expected {}",
                i + 1,
                record.len(),
                expected_columns
            ));
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
    fn test_parse_csv_with_headers() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let config = CsvConfig::default();
        let result = parse_csv(temp_file.path(), &config);
        assert!(result.is_ok());
        
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0], vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_parse_csv_without_headers() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let config = CsvConfig {
            has_headers: false,
            ..CsvConfig::default()
        };
        
        let result = parse_csv(temp_file.path(), &config);
        assert!(result.is_ok());
        
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
    }

    #[test]
    fn test_validate_records() {
        let valid_records = vec![
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec!["d".to_string(), "e".to_string(), "f".to_string()],
        ];
        
        assert!(validate_records(&valid_records, 3).is_ok());
        
        let invalid_records = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string(), "d".to_string(), "e".to_string()],
        ];
        
        assert!(validate_records(&invalid_records, 3).is_err());
    }
}