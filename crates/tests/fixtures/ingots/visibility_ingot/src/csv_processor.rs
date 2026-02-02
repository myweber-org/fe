
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub struct CsvProcessor {
    input_path: String,
    output_path: String,
    filter_column: usize,
    filter_value: String,
    transform_column: usize,
    transform_fn: Box<dyn Fn(&str) -> String>,
}

impl CsvProcessor {
    pub fn new(
        input_path: String,
        output_path: String,
        filter_column: usize,
        filter_value: String,
        transform_column: usize,
        transform_fn: Box<dyn Fn(&str) -> String>,
    ) -> Self {
        CsvProcessor {
            input_path,
            output_path,
            filter_column,
            filter_value,
            transform_column,
            transform_fn,
        }
    }

    pub fn process(&self) -> Result<(), Box<dyn Error>> {
        let input_file = File::open(Path::new(&self.input_path))?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(Path::new(&self.output_path))?;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();

            if line_num == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }

            if parts.len() <= self.filter_column.max(self.transform_column) {
                continue;
            }

            if parts[self.filter_column] != self.filter_value {
                continue;
            }

            let mut transformed_parts = parts.clone();
            transformed_parts[self.transform_column] = &(self.transform_fn)(parts[self.transform_column]);
            
            let transformed_line = transformed_parts.join(",");
            writeln!(output_file, "{}", transformed_line)?;
        }

        Ok(())
    }
}

pub fn create_uppercase_transformer() -> Box<dyn Fn(&str) -> String> {
    Box::new(|s: &str| s.to_uppercase())
}

pub fn create_prefix_transformer(prefix: &str) -> Box<dyn Fn(&str) -> String> {
    let prefix = prefix.to_string();
    Box::new(move |s: &str| format!("{}{}", prefix, s))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_csv_processing() {
        let test_input = "test_input.csv";
        let test_output = "test_output.csv";
        
        let input_content = "id,name,status\n1,alice,active\n2,bob,inactive\n3,charlie,active\n";
        fs::write(test_input, input_content).unwrap();

        let processor = CsvProcessor::new(
            test_input.to_string(),
            test_output.to_string(),
            2,
            "active".to_string(),
            1,
            create_uppercase_transformer(),
        );

        let result = processor.process();
        assert!(result.is_ok());

        let output_content = fs::read_to_string(test_output).unwrap();
        let expected = "id,name,status\n1,ALICE,active\n3,CHARLIE,active\n";
        assert_eq!(output_content, expected);

        fs::remove_file(test_input).unwrap();
        fs::remove_file(test_output).unwrap();
    }
}
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        Ok(())
    }

    fn transform(&mut self) {
        self.name = self.name.to_uppercase();
        self.value = (self.value * 100.0).round() / 100.0;
    }
}

fn process_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(input_file);

    let output_file = File::create(output_path)?;
    let mut writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(output_file);

    for result in reader.deserialize() {
        let mut record: Record = result?;
        
        match record.validate() {
            Ok(_) => {
                record.transform();
                writer.serialize(&record)?;
            }
            Err(e) => eprintln!("Validation failed for record: {} - {}", record.id, e),
        }
    }

    writer.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/output.csv";
    
    process_csv(input_file, output_file)?;
    println!("CSV processing completed successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_record_validation() {
        let valid_record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 42.5,
            active: true,
        };
        assert!(valid_record.validate().is_ok());

        let invalid_record = Record {
            id: 2,
            name: "".to_string(),
            value: -10.0,
            active: false,
        };
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_record_transformation() {
        let mut record = Record {
            id: 1,
            name: "test".to_string(),
            value: 123.456,
            active: true,
        };
        
        record.transform();
        assert_eq!(record.name, "TEST");
        assert_eq!(record.value, 123.46);
    }

    #[test]
    fn test_csv_processing() -> Result<(), Box<dyn Error>> {
        let csv_data = "id,name,value,active\n1,test,42.5,true\n2,another,99.99,false\n";
        
        let mut input_file = NamedTempFile::new()?;
        write!(input_file, "{}", csv_data)?;
        
        let output_file = NamedTempFile::new()?;
        
        process_csv(input_file.path().to_str().unwrap(), output_file.path().to_str().unwrap())?;
        
        Ok(())
    }
}