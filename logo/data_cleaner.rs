use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    age: u32,
    email: String,
}

fn validate_record(record: &Record) -> Result<(), String> {
    if record.name.trim().is_empty() {
        return Err("Name cannot be empty".to_string());
    }
    if record.age > 120 {
        return Err("Age must be less than 120".to_string());
    }
    if !record.email.contains('@') {
        return Err("Invalid email format".to_string());
    }
    Ok(())
}

fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(Path::new(input_path))?;
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(input_file);
    
    let output_file = File::create(Path::new(output_path))?;
    let mut wtr = WriterBuilder::new().has_headers(true).from_writer(output_file);
    
    let mut valid_count = 0;
    let mut invalid_count = 0;
    
    for result in rdr.deserialize() {
        let record: Record = result?;
        
        match validate_record(&record) {
            Ok(_) => {
                wtr.serialize(&record)?;
                valid_count += 1;
            }
            Err(err) => {
                eprintln!("Invalid record ID {}: {}", record.id, err);
                invalid_count += 1;
            }
        }
    }
    
    wtr.flush()?;
    println!("Processing complete: {} valid, {} invalid records", valid_count, invalid_count);
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input = "data/raw_data.csv";
    let output = "data/cleaned_data.csv";
    
    clean_csv(input, output)?;
    
    Ok(())
}
use std::collections::HashMap;

pub fn filter_numeric_data(data: &HashMap<String, String>) -> HashMap<String, f64> {
    let mut numeric_map = HashMap::new();

    for (key, value) in data {
        if let Ok(parsed_value) = value.parse::<f64>() {
            numeric_map.insert(key.clone(), parsed_value);
        }
    }

    numeric_map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_numeric_data() {
        let mut test_data = HashMap::new();
        test_data.insert("age".to_string(), "25".to_string());
        test_data.insert("name".to_string(), "Alice".to_string());
        test_data.insert("height".to_string(), "1.75".to_string());
        test_data.insert("city".to_string(), "London".to_string());

        let result = filter_numeric_data(&test_data);

        assert_eq!(result.len(), 2);
        assert_eq!(result.get("age"), Some(&25.0));
        assert_eq!(result.get("height"), Some(&1.75));
        assert_eq!(result.get("name"), None);
    }
}