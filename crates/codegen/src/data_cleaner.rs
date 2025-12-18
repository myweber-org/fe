use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn clean_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = Reader::from_reader(input_file);
    let mut valid_records = Vec::new();
    let mut invalid_count = 0;

    for result in reader.deserialize() {
        let record: Record = match result {
            Ok(rec) => rec,
            Err(e) => {
                eprintln!("Invalid record skipped: {}", e);
                invalid_count += 1;
                continue;
            }
        };

        if record.value >= 0.0 && !record.name.is_empty() {
            valid_records.push(record);
        } else {
            invalid_count += 1;
        }
    }

    if !valid_records.is_empty() {
        let output_file = File::create(output_path)?;
        let mut writer = csv::Writer::from_writer(output_file);
        
        for record in valid_records {
            writer.serialize(record)?;
        }
        
        writer.flush()?;
        println!("Cleaned data saved to: {}", output_path);
    }

    println!("Processing complete. Valid records: {}, Invalid records: {}", 
             valid_records.len(), invalid_count);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_clean_csv_data() {
        let mut input_file = NamedTempFile::new().unwrap();
        writeln!(input_file, "id,name,value,category").unwrap();
        writeln!(input_file, "1,ItemA,25.5,Category1").unwrap();
        writeln!(input_file, "2,,15.0,Category2").unwrap();
        writeln!(input_file, "3,ItemC,-5.0,Category1").unwrap();

        let output_file = NamedTempFile::new().unwrap();
        
        let result = clean_csv_data(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap()
        );
        
        assert!(result.is_ok());
    }
}