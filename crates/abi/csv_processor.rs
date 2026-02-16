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
            if record.get(column_index) == Some(target_value) {
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
    
    #[test]
    fn test_filter_by_column_value() {
        let input_data = "name,age,city\nAlice,30,London\nBob,25,Paris\nCharlie,35,London";
        let input_path = "test_input.csv";
        let output_path = "test_output.csv";
        
        fs::write(input_path, input_data).unwrap();
        
        let processor = CsvProcessor::new(input_path, output_path);
        let result = processor.filter_by_column_value("city", "London");
        
        assert!(result.is_ok());
        
        let output = fs::read_to_string(output_path).unwrap();
        let expected = "name,age,city\nAlice,30,London\nCharlie,35,London\n";
        assert_eq!(output, expected);
        
        fs::remove_file(input_path).unwrap();
        fs::remove_file(output_path).unwrap();
    }
}