use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

pub struct CsvProcessor {
    input_path: String,
    output_path: String,
    filter_column: usize,
    filter_value: String,
}

impl CsvProcessor {
    pub fn new(input_path: &str, output_path: &str, filter_column: usize, filter_value: &str) -> Self {
        CsvProcessor {
            input_path: input_path.to_string(),
            output_path: output_path.to_string(),
            filter_column,
            filter_value: filter_value.to_string(),
        }
    }

    pub fn process(&self) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(&self.output_path)?;
        let mut processed_count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();

            if line_num == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }

            if parts.get(self.filter_column).map(|&val| val == self.filter_value).unwrap_or(false) {
                let transformed_line = self.transform_record(&parts);
                writeln!(output_file, "{}", transformed_line)?;
                processed_count += 1;
            }
        }

        Ok(processed_count)
    }

    fn transform_record(&self, record: &[&str]) -> String {
        let mut transformed: Vec<String> = record.iter().map(|&s| s.to_uppercase()).collect();
        transformed.push("PROCESSED".to_string());
        transformed.join(",")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn test_csv_processing() {
        let test_input = "id,name,status\n1,alice,active\n2,bob,inactive\n3,charlie,active";
        let input_path = "test_input.csv";
        let output_path = "test_output.csv";
        
        let mut input_file = File::create(input_path).unwrap();
        input_file.write_all(test_input.as_bytes()).unwrap();

        let processor = CsvProcessor::new(input_path, output_path, 2, "active");
        let result = processor.process();

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);

        let mut output_file = File::open(output_path).unwrap();
        let mut output_content = String::new();
        output_file.read_to_string(&mut output_content).unwrap();

        assert!(output_content.contains("ALICE"));
        assert!(output_content.contains("CHARLIE"));
        assert!(!output_content.contains("BOB"));

        std::fs::remove_file(input_path).unwrap();
        std::fs::remove_file(output_path).unwrap();
    }
}
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    email: String,
    active: bool,
}

fn validate_email(email: &str) -> bool {
    email.contains('@') && email.contains('.')
}

fn clean_name(name: &str) -> String {
    name.trim().to_string()
}

fn process_csv(input_path: &Path, output_path: &Path) -> Result<(), Box<dyn Error>> {
    let file = File::open(input_path)?;
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file);

    let output_file = File::create(output_path)?;
    let mut wtr = WriterBuilder::new()
        .has_headers(true)
        .from_writer(output_file);

    for result in rdr.deserialize() {
        let mut record: Record = result?;
        
        record.name = clean_name(&record.name);
        
        if !validate_email(&record.email) {
            eprintln!("Invalid email for record {}: {}", record.id, record.email);
            continue;
        }

        wtr.serialize(&record)?;
    }

    wtr.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input = Path::new("data/input.csv");
    let output = Path::new("data/cleaned_output.csv");
    
    process_csv(input, output)?;
    println!("CSV processing completed successfully");
    Ok(())
}