use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn clean_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(Path::new(input_path))?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(input_file);

    let output_file = File::create(Path::new(output_path))?;
    let mut writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(output_file);

    for result in reader.deserialize() {
        let record: Record = result?;
        
        let cleaned_record = Record {
            id: record.id,
            name: record.name.trim().to_string(),
            value: if record.value.is_nan() || record.value.is_infinite() {
                0.0
            } else {
                record.value
            },
            category: if record.category.is_empty() {
                "unknown".to_string()
            } else {
                record.category.to_lowercase()
            },
        };

        writer.serialize(&cleaned_record)?;
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
    fn test_clean_csv_data() {
        let mut input_file = NamedTempFile::new().unwrap();
        writeln!(input_file, "id,name,value,category").unwrap();
        writeln!(input_file, "1,  John Doe ,42.5,TECH").unwrap();
        writeln!(input_file, "2,Jane Smith,NaN,").unwrap();

        let output_file = NamedTempFile::new().unwrap();

        clean_csv_data(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
        ).unwrap();

        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(File::open(output_file.path()).unwrap());

        let records: Vec<Record> = reader.deserialize().collect::<Result<_, _>>().unwrap();
        
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "John Doe");
        assert_eq!(records[0].category, "tech");
        assert_eq!(records[1].value, 0.0);
        assert_eq!(records[1].category, "unknown");
    }
}