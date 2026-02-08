
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    age: u8,
    active: bool,
}

fn clean_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(Path::new(input_path))?;
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(input_file);

    let output_file = File::create(Path::new(output_path))?;
    let mut wtr = WriterBuilder::new()
        .has_headers(true)
        .from_writer(output_file);

    for result in rdr.deserialize() {
        let mut record: Record = result?;
        
        // Data cleaning operations
        record.name = record.name.trim().to_string();
        if record.name.is_empty() {
            record.name = "Unknown".to_string();
        }
        
        if record.age > 120 {
            record.age = 120;
        }
        
        wtr.serialize(&record)?;
    }

    wtr.flush()?;
    Ok(())
}

fn validate_record(record: &Record) -> bool {
    !record.name.is_empty() && record.age > 0 && record.age <= 120
}

fn main() -> Result<(), Box<dyn Error>> {
    let input = "data/raw.csv";
    let output = "data/cleaned.csv";
    
    match clean_csv_data(input, output) {
        Ok(_) => println!("Data cleaning completed successfully"),
        Err(e) => eprintln!("Error during data cleaning: {}", e),
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_clean_csv_data() {
        let input_data = "id,name,age,active\n1,  John Doe  ,30,true\n2,,150,false\n";
        
        let mut input_file = NamedTempFile::new().unwrap();
        write!(input_file, "{}", input_data).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        let result = clean_csv_data(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap()
        );
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_record() {
        let valid_record = Record {
            id: 1,
            name: "Test".to_string(),
            age: 25,
            active: true,
        };
        
        let invalid_record = Record {
            id: 2,
            name: "".to_string(),
            age: 0,
            active: false,
        };
        
        assert!(validate_record(&valid_record));
        assert!(!validate_record(&invalid_record));
    }
}