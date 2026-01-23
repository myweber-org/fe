use csv::{Reader, Writer};
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

pub fn process_data(input_path: &str, output_path: &str, category_filter: Option<&str>) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut writer = Writer::from_path(output_path)?;

    for result in reader.deserialize() {
        let mut record: Record = result?;
        
        if let Some(filter) = category_filter {
            if record.category != filter {
                continue;
            }
        }

        record.value = apply_transform(record.value);
        
        writer.serialize(&record)?;
    }

    writer.flush()?;
    Ok(())
}

fn apply_transform(value: f64) -> f64 {
    (value * 1.1).round()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let input_data = "id,name,value,category\n1,test1,100.0,A\n2,test2,200.0,B";
        let input_file = NamedTempFile::new().unwrap();
        std::fs::write(input_file.path(), input_data).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        let result = process_data(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            Some("A")
        );
        
        assert!(result.is_ok());
        
        let output_content = std::fs::read_to_string(output_file.path()).unwrap();
        assert!(output_content.contains("test1"));
        assert!(!output_content.contains("test2"));
    }
}