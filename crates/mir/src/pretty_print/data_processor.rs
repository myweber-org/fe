use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

struct DataProcessor {
    input_path: String,
    output_path: String,
}

impl DataProcessor {
    fn new(input_path: &str, output_path: &str) -> Self {
        DataProcessor {
            input_path: input_path.to_string(),
            output_path: output_path.to_string(),
        }
    }

    fn process_data(&self, min_value: f64) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(reader);

        let output_file = File::create(&self.output_path)?;
        let writer = BufWriter::new(output_file);
        let mut csv_writer = WriterBuilder::new()
            .has_headers(true)
            .from_writer(writer);

        let mut processed_count = 0;

        for result in csv_reader.deserialize() {
            let record: Record = result?;
            
            if record.value >= min_value && record.active {
                csv_writer.serialize(&record)?;
                processed_count += 1;
            }
        }

        csv_writer.flush()?;
        Ok(processed_count)
    }

    fn calculate_statistics(&self) -> Result<(f64, f64, usize), Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(reader);

        let mut total = 0.0;
        let mut count = 0;
        let mut max_value = f64::MIN;

        for result in csv_reader.deserialize() {
            let record: Record = result?;
            
            if record.active {
                total += record.value;
                count += 1;
                if record.value > max_value {
                    max_value = record.value;
                }
            }
        }

        let average = if count > 0 { total / count as f64 } else { 0.0 };
        Ok((average, max_value, count))
    }
}

fn validate_file_path(path: &str) -> bool {
    Path::new(path).exists()
}

pub fn run_processing(input_path: &str, output_path: &str, threshold: f64) -> Result<(), Box<dyn Error>> {
    if !validate_file_path(input_path) {
        return Err("Input file does not exist".into());
    }

    let processor = DataProcessor::new(input_path, output_path);
    
    match processor.process_data(threshold) {
        Ok(count) => {
            println!("Processed {} records", count);
            
            match processor.calculate_statistics() {
                Ok((avg, max, total)) => {
                    println!("Statistics - Average: {:.2}, Max: {:.2}, Total active records: {}", avg, max, total);
                }
                Err(e) => eprintln!("Failed to calculate statistics: {}", e),
            }
        }
        Err(e) => eprintln!("Processing failed: {}", e),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_data_processing() {
        let input_data = "id,name,category,value,active\n\
                          1,ItemA,Category1,25.5,true\n\
                          2,ItemB,Category2,15.0,false\n\
                          3,ItemC,Category1,30.0,true\n";

        let mut input_file = NamedTempFile::new().unwrap();
        write!(input_file, "{}", input_data).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        let processor = DataProcessor::new(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap()
        );
        
        let result = processor.process_data(20.0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
    }
}