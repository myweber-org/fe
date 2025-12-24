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

            if parts.get(self.filter_column).map(|&v| v.trim()) == Some(self.filter_value.trim()) {
                let transformed_line = parts.iter()
                    .map(|field| field.trim().to_uppercase())
                    .collect::<Vec<String>>()
                    .join(",");
                writeln!(output_file, "{}", transformed_line)?;
                processed_count += 1;
            }
        }

        Ok(processed_count)
    }

    pub fn validate_config(&self) -> Result<(), String> {
        if self.filter_column > 10 {
            return Err("Filter column index too large".to_string());
        }
        if self.input_path.is_empty() || self.output_path.is_empty() {
            return Err("File paths cannot be empty".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn test_csv_processing() {
        let test_input = "id,name,status\n1,alice,active\n2,bob,inactive\n3,charlie,active";
        let temp_input = "test_input.csv";
        let temp_output = "test_output.csv";
        
        std::fs::write(temp_input, test_input).unwrap();
        
        let processor = CsvProcessor::new(temp_input, temp_output, 2, "active");
        processor.validate_config().unwrap();
        let result = processor.process().unwrap();
        
        assert_eq!(result, 2);
        
        let mut output_content = String::new();
        File::open(temp_output).unwrap().read_to_string(&mut output_content).unwrap();
        assert!(output_content.contains("1,ALICE,ACTIVE"));
        assert!(output_content.contains("3,CHARLIE,ACTIVE"));
        
        std::fs::remove_file(temp_input).unwrap();
        std::fs::remove_file(temp_output).unwrap();
    }
}