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

fn clean_numeric_field(value: f64, threshold: f64) -> Option<f64> {
    if value.is_finite() && value.abs() <= threshold {
        Some(value)
    } else {
        None
    }
}

fn validate_category(category: &str) -> bool {
    let valid_categories = ["A", "B", "C", "D"];
    valid_categories.contains(&category)
}

pub fn process_csv_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(input_path)?;
    let mut reader = Reader::from_reader(file);
    let mut valid_records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if let Some(cleaned_value) = clean_numeric_field(record.value, 1000.0) {
            if validate_category(&record.category) {
                valid_records.push((record.id, record.name, cleaned_value, record.category));
            }
        }
    }

    let output_file = File::create(output_path)?;
    let mut writer = csv::Writer::from_writer(output_file);
    
    for (id, name, value, category) in valid_records {
        writer.write_record(&[
            id.to_string(),
            name,
            value.to_string(),
            category,
        ])?;
    }
    
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_clean_numeric_field() {
        assert_eq!(clean_numeric_field(42.5, 1000.0), Some(42.5));
        assert_eq!(clean_numeric_field(f64::INFINITY, 1000.0), None);
        assert_eq!(clean_numeric_field(1500.0, 1000.0), None);
    }

    #[test]
    fn test_validate_category() {
        assert!(validate_category("A"));
        assert!(validate_category("B"));
        assert!(!validate_category("X"));
    }

    #[test]
    fn test_process_csv() -> Result<(), Box<dyn Error>> {
        let mut temp_input = NamedTempFile::new()?;
        writeln!(temp_input, "id,name,value,category")?;
        writeln!(temp_input, "1,Test1,100.5,A")?;
        writeln!(temp_input, "2,Test2,2000.0,B")?;
        
        let temp_output = NamedTempFile::new()?;
        
        process_csv_file(
            temp_input.path().to_str().unwrap(),
            temp_output.path().to_str().unwrap(),
        )?;
        
        Ok(())
    }
}