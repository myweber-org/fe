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
    email: String,
}

fn clean_email(email: &str) -> String {
    email.trim().to_lowercase()
}

fn validate_age(age: u8) -> Option<u8> {
    if age > 0 && age < 120 {
        Some(age)
    } else {
        None
    }
}

pub fn clean_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(Path::new(input_path))?;
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(input_file);
    
    let output_file = File::create(Path::new(output_path))?;
    let mut wtr = WriterBuilder::new().has_headers(true).from_writer(output_file);

    for result in rdr.deserialize() {
        let mut record: Record = result?;
        
        record.email = clean_email(&record.email);
        record.name = record.name.trim().to_string();
        
        if let Some(valid_age) = validate_age(record.age) {
            record.age = valid_age;
            wtr.serialize(&record)?;
        }
    }
    
    wtr.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_clean_csv_data() {
        let input_data = "id,name,age,email\n1, John Doe ,25, JOHN@EXAMPLE.COM\n2,Jane Smith,150,jane@example.com\n";
        let mut input_file = NamedTempFile::new().unwrap();
        write!(input_file, "{}", input_data).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        let result = clean_csv_data(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap()
        );
        
        assert!(result.is_ok());
        
        let output_content = std::fs::read_to_string(output_file.path()).unwrap();
        assert!(output_content.contains("john@example.com"));
        assert!(!output_content.contains("150"));
    }
}