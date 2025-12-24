use csv::{ReaderBuilder, WriterBuilder};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};

pub fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);
    
    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    let mut csv_writer = WriterBuilder::new().has_headers(true).from_writer(writer);
    
    let headers = csv_reader.headers()?.clone();
    csv_writer.write_record(&headers)?;
    
    for result in csv_reader.records() {
        let record = result?;
        let filtered_record: Vec<&str> = record
            .iter()
            .filter(|field| !field.trim().is_empty())
            .collect();
        
        if filtered_record.len() == headers.len() {
            csv_writer.write_record(&filtered_record)?;
        }
    }
    
    csv_writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_clean_csv() {
        let input_data = "name,age,city\nJohn,25,NYC\nJane,,London\nBob,30,\n,40,Boston";
        let input_temp = NamedTempFile::new().unwrap();
        fs::write(input_temp.path(), input_data).unwrap();
        
        let output_temp = NamedTempFile::new().unwrap();
        
        clean_csv(
            input_temp.path().to_str().unwrap(),
            output_temp.path().to_str().unwrap()
        ).unwrap();
        
        let output = fs::read_to_string(output_temp.path()).unwrap();
        assert_eq!(output, "name,age,city\nJohn,25,NYC\n");
    }
}use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

fn clean_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = Reader::from_reader(input_file);
    
    let output_file = File::create(output_path)?;
    let mut writer = Writer::from_writer(output_file);

    for result in reader.deserialize() {
        let mut record: Record = result?;
        
        record.name = record.name.trim().to_string();
        record.category = record.category.to_uppercase();
        
        if record.value < 0.0 {
            record.value = 0.0;
        }
        
        if record.name.is_empty() {
            continue;
        }
        
        writer.serialize(&record)?;
    }

    writer.flush()?;
    Ok(())
}

fn validate_record(record: &Record) -> bool {
    !record.name.is_empty() && record.value >= 0.0
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "input_data.csv";
    let output_file = "cleaned_data.csv";
    
    match clean_csv_data(input_file, output_file) {
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
        let input_data = "id,name,value,category\n1,test,10.5,category1\n2,negative,-5.0,category2\n";
        
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
            name: "test".to_string(),
            value: 10.0,
            category: "CATEGORY".to_string(),
        };
        
        let invalid_record = Record {
            id: 2,
            name: "".to_string(),
            value: -5.0,
            category: "CATEGORY".to_string(),
        };
        
        assert!(validate_record(&valid_record));
        assert!(!validate_record(&invalid_record));
    }
}