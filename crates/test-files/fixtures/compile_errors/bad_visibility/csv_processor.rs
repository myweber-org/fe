use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use csv::{ReaderBuilder, WriterBuilder};

pub struct CsvProcessor {
    input_path: String,
    output_path: String,
}

impl CsvProcessor {
    pub fn new(input_path: &str, output_path: &str) -> Self {
        CsvProcessor {
            input_path: input_path.to_string(),
            output_path: output_path.to_string(),
        }
    }

    pub fn filter_by_column_value(&self, column_name: &str, target_value: &str) -> Result<(), Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);
        
        let output_file = File::create(&self.output_path)?;
        let writer = BufWriter::new(output_file);
        let mut csv_writer = WriterBuilder::new().from_writer(writer);
        
        let headers = csv_reader.headers()?.clone();
        csv_writer.write_record(&headers)?;
        
        let column_index = headers.iter()
            .position(|h| h == column_name)
            .ok_or_else(|| format!("Column '{}' not found", column_name))?;
        
        for result in csv_reader.records() {
            let record = result?;
            if record.get(column_index).map(|v| v == target_value).unwrap_or(false) {
                csv_writer.write_record(&record)?;
            }
        }
        
        csv_writer.flush()?;
        Ok(())
    }

    pub fn transform_column(&self, column_name: &str, transform_fn: fn(&str) -> String) -> Result<(), Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);
        
        let output_file = File::create(&self.output_path)?;
        let writer = BufWriter::new(output_file);
        let mut csv_writer = WriterBuilder::new().from_writer(writer);
        
        let headers = csv_reader.headers()?.clone();
        csv_writer.write_record(&headers)?;
        
        let column_index = headers.iter()
            .position(|h| h == column_name)
            .ok_or_else(|| format!("Column '{}' not found", column_name))?;
        
        for result in csv_reader.records() {
            let mut record = result?.clone();
            if let Some(value) = record.get(column_index) {
                let transformed = transform_fn(value);
                record[column_index] = transformed.into();
            }
            csv_writer.write_record(&record)?;
        }
        
        csv_writer.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_filter_by_column_value() {
        let input_data = "name,age,city\nAlice,30,London\nBob,25,Paris\nCharlie,35,London";
        let input_file = NamedTempFile::new().unwrap();
        fs::write(input_file.path(), input_data).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        let processor = CsvProcessor::new(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap()
        );
        
        processor.filter_by_column_value("city", "London").unwrap();
        
        let output = fs::read_to_string(output_file.path()).unwrap();
        assert!(output.contains("Alice,30,London"));
        assert!(output.contains("Charlie,35,London"));
        assert!(!output.contains("Bob,25,Paris"));
    }

    #[test]
    fn test_transform_column() {
        let input_data = "name,score\nAlice,85\nBob,92";
        let input_file = NamedTempFile::new().unwrap();
        fs::write(input_file.path(), input_data).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        let processor = CsvProcessor::new(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap()
        );
        
        fn add_percentage(value: &str) -> String {
            format!("{}%", value)
        }
        
        processor.transform_column("score", add_percentage).unwrap();
        
        let output = fs::read_to_string(output_file.path()).unwrap();
        assert!(output.contains("Alice,85%"));
        assert!(output.contains("Bob,92%"));
    }
}