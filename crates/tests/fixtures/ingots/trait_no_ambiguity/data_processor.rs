use csv::{Reader, Writer};
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
    fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0
    }
}

fn process_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = Reader::from_reader(input_file);
    
    let output_file = File::create(output_path)?;
    let mut writer = Writer::from_writer(output_file);

    let mut valid_count = 0;
    let mut invalid_count = 0;

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.is_valid() {
            writer.serialize(&record)?;
            valid_count += 1;
        } else {
            invalid_count += 1;
        }
    }

    writer.flush()?;
    
    println!("Processing complete:");
    println!("  Valid records: {}", valid_count);
    println!("  Invalid records: {}", invalid_count);
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "input_data.csv";
    let output_file = "processed_data.csv";
    
    match process_csv(input_file, output_file) {
        Ok(_) => println!("Data processing completed successfully"),
        Err(e) => eprintln!("Error processing data: {}", e),
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_valid_record() {
        let record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 10.5,
            active: true,
        };
        assert!(record.is_valid());
    }

    #[test]
    fn test_invalid_record() {
        let empty_name = Record {
            id: 2,
            name: "".to_string(),
            value: 5.0,
            active: false,
        };
        assert!(!empty_name.is_valid());

        let negative_value = Record {
            id: 3,
            name: "Test".to_string(),
            value: -1.0,
            active: true,
        };
        assert!(!negative_value.is_valid());
    }
}