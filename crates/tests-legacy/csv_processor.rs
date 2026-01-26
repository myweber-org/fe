use std::error::Error;
use std::fs::File;
use csv::{ReaderBuilder, WriterBuilder};

pub struct CsvProcessor {
    input_path: String,
    output_path: String,
}

impl CsvProcessor {
    pub fn new(input: &str, output: &str) -> Self {
        CsvProcessor {
            input_path: input.to_string(),
            output_path: output.to_string(),
        }
    }

    pub fn filter_by_column_value(&self, column_name: &str, filter_value: &str) -> Result<(), Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(input_file);
        
        let output_file = File::create(&self.output_path)?;
        let mut wtr = WriterBuilder::new().has_headers(true).from_writer(output_file);

        let headers = rdr.headers()?.clone();
        let column_index = headers.iter()
            .position(|h| h == column_name)
            .ok_or_else(|| format!("Column '{}' not found", column_name))?;

        wtr.write_record(&headers)?;

        for result in rdr.records() {
            let record = result?;
            if record.get(column_index).map(|v| v == filter_value).unwrap_or(false) {
                wtr.write_record(&record)?;
            }
        }

        wtr.flush()?;
        Ok(())
    }

    pub fn transform_column(&self, column_name: &str, transform_fn: fn(&str) -> String) -> Result<(), Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(input_file);
        
        let output_file = File::create(&self.output_path)?;
        let mut wtr = WriterBuilder::new().has_headers(true).from_writer(output_file);

        let headers = rdr.headers()?.clone();
        let column_index = headers.iter()
            .position(|h| h == column_name)
            .ok_or_else(|| format!("Column '{}' not found", column_name))?;

        wtr.write_record(&headers)?;

        for result in rdr.records() {
            let mut record = result?.clone();
            if let Some(value) = record.get(column_index) {
                let transformed = transform_fn(value);
                record[column_index] = transformed.into();
            }
            wtr.write_record(&record)?;
        }

        wtr.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "name,age,city").unwrap();
        writeln!(file, "Alice,30,New York").unwrap();
        writeln!(file, "Bob,25,London").unwrap();
        writeln!(file, "Charlie,35,New York").unwrap();
        file
    }

    #[test]
    fn test_filter_by_column() {
        let input_file = create_test_csv();
        let output_file = NamedTempFile::new().unwrap();
        
        let processor = CsvProcessor::new(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap()
        );
        
        processor.filter_by_column_value("city", "New York").unwrap();
        
        let content = std::fs::read_to_string(output_file.path()).unwrap();
        assert!(content.contains("Alice"));
        assert!(!content.contains("Bob"));
        assert!(content.contains("Charlie"));
    }

    #[test]
    fn test_transform_column() {
        let input_file = create_test_csv();
        let output_file = NamedTempFile::new().unwrap();
        
        let processor = CsvProcessor::new(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap()
        );
        
        fn uppercase(s: &str) -> String {
            s.to_uppercase()
        }
        
        processor.transform_column("city", uppercase).unwrap();
        
        let content = std::fs::read_to_string(output_file.path()).unwrap();
        assert!(content.contains("NEW YORK"));
        assert!(content.contains("LONDON"));
    }
}