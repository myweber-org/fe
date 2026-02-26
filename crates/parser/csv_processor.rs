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
    pub fn new(input: &str, output: &str, column: usize, value: &str) -> Self {
        CsvProcessor {
            input_path: input.to_string(),
            output_path: output.to_string(),
            filter_column,
            filter_value: value.to_string(),
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

            if parts.get(self.filter_column)
                .map(|val| val.trim() == self.filter_value)
                .unwrap_or(false)
            {
                let transformed = parts.iter()
                    .map(|s| s.trim().to_uppercase())
                    .collect::<Vec<String>>()
                    .join(",");
                writeln!(output_file, "{}", transformed)?;
                processed_count += 1;
            }
        }

        Ok(processed_count)
    }

    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        if self.filter_column > 10 {
            return Err("Filter column index too large".into());
        }
        if self.input_path == self.output_path {
            return Err("Input and output paths must differ".into());
        }
        Ok(())
    }
}

pub fn run_processing(input: &str, output: &str, column: usize, value: &str) -> Result<usize, Box<dyn Error>> {
    let processor = CsvProcessor::new(input, output, column, value);
    processor.validate()?;
    processor.process()
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

pub struct CsvProcessor {
    input_path: String,
    output_path: String,
    filter_column: usize,
    filter_value: String,
}

impl CsvProcessor {
    pub fn new(input: &str, output: &str, column: usize, value: &str) -> Self {
        CsvProcessor {
            input_path: input.to_string(),
            output_path: output.to_string(),
            filter_column,
            filter_value: value.to_string(),
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

            if parts.get(self.filter_column)
                .map(|val| val.trim() == self.filter_value)
                .unwrap_or(false)
            {
                let transformed = parts.iter()
                    .map(|s| s.to_uppercase())
                    .collect::<Vec<String>>()
                    .join(",");
                writeln!(output_file, "{}", transformed)?;
                processed_count += 1;
            }
        }

        Ok(processed_count)
    }
}

pub fn run_example() -> Result<(), Box<dyn Error>> {
    let processor = CsvProcessor::new(
        "input_data.csv",
        "filtered_output.csv",
        2,
        "active"
    );
    
    match processor.process() {
        Ok(count) => println!("Processed {} matching records", count),
        Err(e) => eprintln!("Processing failed: {}", e),
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    
    #[test]
    fn test_csv_processing() {
        let test_input = "id,name,status\n1,alice,active\n2,bob,inactive\n3,charlie,active";
        let mut temp_input = tempfile::NamedTempFile::new().unwrap();
        temp_input.write_all(test_input.as_bytes()).unwrap();
        
        let temp_output = tempfile::NamedTempFile::new().unwrap();
        
        let processor = CsvProcessor::new(
            temp_input.path().to_str().unwrap(),
            temp_output.path().to_str().unwrap(),
            2,
            "active"
        );
        
        let result = processor.process();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
        
        let mut output_content = String::new();
        File::open(temp_output.path())
            .unwrap()
            .read_to_string(&mut output_content)
            .unwrap();
        
        assert!(output_content.contains("ALICE"));
        assert!(!output_content.contains("BOB"));
        assert!(output_content.contains("CHARLIE"));
    }
}