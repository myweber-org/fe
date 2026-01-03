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

            if let Some(value) = parts.get(self.filter_column) {
                if value.trim() == self.filter_value {
                    writeln!(output_file, "{}", line)?;
                    processed_count += 1;
                }
            }
        }

        Ok(processed_count)
    }

    pub fn transform_column(&self, column: usize, transformer: fn(&str) -> String) -> Result<(), Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(&self.output_path)?;
        let mut lines = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let mut parts: Vec<String> = line.split(',').map(|s| s.to_string()).collect();
            
            if line_num == 0 {
                lines.push(line);
                continue;
            }

            if let Some(value) = parts.get_mut(column) {
                *value = transformer(value);
            }
            lines.push(parts.join(","));
        }

        for line in lines {
            writeln!(output_file, "{}", line)?;
        }

        Ok(())
    }
}

pub fn uppercase_transform(value: &str) -> String {
    value.to_uppercase()
}

pub fn lowercase_transform(value: &str) -> String {
    value.to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_csv_processing() {
        let test_data = "id,name,value\n1,test,100\n2,demo,200\n3,test,300";
        fs::write("test_input.csv", test_data).unwrap();

        let processor = CsvProcessor::new("test_input.csv", "test_output.csv", 1, "test");
        let result = processor.process().unwrap();
        
        assert_eq!(result, 2);
        
        let output = fs::read_to_string("test_output.csv").unwrap();
        assert!(output.contains("1,test,100"));
        assert!(output.contains("3,test,300"));
        
        fs::remove_file("test_input.csv").unwrap();
        fs::remove_file("test_output.csv").unwrap();
    }
}