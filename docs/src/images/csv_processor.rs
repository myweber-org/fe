
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

impl From<std::io::Error> for CsvError {
    fn from(error: std::io::Error) -> Self {
        CsvError::IoError(error.to_string())
    }
}

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
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }

            let record = self.parse_line(&line, line_num + 1)?;
            self.validate_record(&record, line_num + 1)?;
            self.records.push(record);
        }

        Ok(())
    }

    fn parse_line(&self, line: &str, line_num: usize) -> Result<CsvRecord, CsvError> {
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        
        if parts.len() != 4 {
            return Err(CsvError::ParseError(
                format!("Line {}: Expected 4 fields, found {}", line_num, parts.len())
            ));
        }

        let id = parts[0].parse::<u32>()
            .map_err(|e| CsvError::ParseError(format!("Line {}: Invalid ID: {}", line_num, e)))?;
        
        let name = parts[1].to_string();
        
        let value = parts[2].parse::<f64>()
            .map_err(|e| CsvError::ParseError(format!("Line {}: Invalid value: {}", line_num, e)))?;
        
        let active = parts[3].parse::<bool>()
            .map_err(|e| CsvError::ParseError(format!("Line {}: Invalid active flag: {}", line_num, e)))?;

        Ok(CsvRecord {
            id,
            name,
            value,
            active,
        })
    }

    fn validate_record(&self, record: &CsvRecord, line_num: usize) -> Result<(), CsvError> {
        if record.name.is_empty() {
            return Err(CsvError::ValidationError(
                format!("Line {}: Name cannot be empty", line_num)
            ));
        }

        if record.value < 0.0 {
            return Err(CsvError::ValidationError(
                format!("Line {}: Value cannot be negative", line_num)
            ));
        }

        Ok(())
    }

    pub fn get_records(&self) -> &[CsvRecord] {
        &self.records
    }

    pub fn filter_active(&self) -> Vec<&CsvRecord> {
        self.records.iter()
            .filter(|r| r.active)
            .collect()
    }

    pub fn calculate_total(&self) -> f64 {
        self.records.iter()
            .map(|r| r.value)
            .sum()
    }

    pub fn find_by_id(&self, id: u32) -> Option<&CsvRecord> {
        self.records.iter()
            .find(|r| r.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_parsing() {
        let mut csv_data = NamedTempFile::new().unwrap();
        writeln!(csv_data, "1,Alice,100.5,true").unwrap();
        writeln!(csv_data, "2,Bob,200.0,false").unwrap();
        writeln!(csv_data, "# This is a comment").unwrap();
        writeln!(csv_data, "").unwrap();
        writeln!(csv_data, "3,Charlie,300.75,true").unwrap();

        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(csv_data.path());
        
        assert!(result.is_ok());
        assert_eq!(processor.records.len(), 3);
        assert_eq!(processor.calculate_total(), 601.25);
        assert_eq!(processor.filter_active().len(), 2);
    }

    #[test]
    fn test_invalid_csv() {
        let mut csv_data = NamedTempFile::new().unwrap();
        writeln!(csv_data, "1,Alice,invalid,true").unwrap();

        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(csv_data.path());
        
        assert!(result.is_err());
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

pub fn filter_csv_rows(
    input_path: &str,
    output_path: &str,
    predicate: impl Fn(&[String]) -> bool,
) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut output_file = File::create(output_path)?;

    for line in reader.lines() {
        let line = line?;
        let fields: Vec<String> = line.split(',').map(|s| s.to_string()).collect();

        if predicate(&fields) {
            writeln!(output_file, "{}", line)?;
        }
    }

    Ok(())
}

pub fn transform_csv_column(
    input_path: &str,
    output_path: &str,
    column_index: usize,
    transformer: impl Fn(&str) -> String,
) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut output_file = File::create(output_path)?;

    for line in reader.lines() {
        let line = line?;
        let mut fields: Vec<String> = line.split(',').map(|s| s.to_string()).collect();

        if column_index < fields.len() {
            fields[column_index] = transformer(&fields[column_index]);
        }

        writeln!(output_file, "{}", fields.join(","))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn test_filter_csv_rows() {
        let input = "data/test_input.csv";
        let output = "data/test_filter_output.csv";
        let test_data = "name,age,city\nAlice,30,NYC\nBob,25,LA\nCharlie,35,Chicago";

        std::fs::create_dir_all("data").unwrap();
        std::fs::write(input, test_data).unwrap();

        let predicate = |fields: &[String]| fields[1].parse::<i32>().unwrap_or(0) > 30;
        filter_csv_rows(input, output, predicate).unwrap();

        let mut content = String::new();
        File::open(output).unwrap().read_to_string(&mut content).unwrap();
        assert_eq!(content.trim(), "Charlie,35,Chicago");

        std::fs::remove_file(input).unwrap();
        std::fs::remove_file(output).unwrap();
    }

    #[test]
    fn test_transform_csv_column() {
        let input = "data/test_transform_input.csv";
        let output = "data/test_transform_output.csv";
        let test_data = "product,price\napple,1.99\nbanana,0.79";

        std::fs::create_dir_all("data").unwrap();
        std::fs::write(input, test_data).unwrap();

        let transformer = |price: &str| {
            let value: f64 = price.parse().unwrap_or(0.0);
            format!("${:.2}", value * 1.1)
        };
        transform_csv_column(input, output, 1, transformer).unwrap();

        let mut content = String::new();
        File::open(output).unwrap().read_to_string(&mut content).unwrap();
        assert!(content.contains("$2.19"));
        assert!(content.contains("$0.87"));

        std::fs::remove_file(input).unwrap();
        std::fs::remove_file(output).unwrap();
    }
}