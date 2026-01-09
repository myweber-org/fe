use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

pub struct DataCleaner {
    input_path: String,
    output_path: String,
}

impl DataCleaner {
    pub fn new(input_path: &str, output_path: &str) -> Self {
        DataCleaner {
            input_path: input_path.to_string(),
            output_path: output_path.to_string(),
        }
    }

    pub fn clean_csv(&self) -> Result<(), Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(&self.output_path)?;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }

            let cleaned_line = self.process_line(&line);
            if !cleaned_line.is_empty() {
                writeln!(output_file, "{}", cleaned_line)?;
            }
        }

        Ok(())
    }

    fn process_line(&self, line: &str) -> String {
        let parts: Vec<&str> = line.split(',').collect();
        let mut cleaned_parts = Vec::new();

        for part in parts {
            let trimmed = part.trim();
            if !trimmed.is_empty() && trimmed != "null" && trimmed != "NULL" {
                cleaned_parts.push(trimmed);
            } else {
                cleaned_parts.push("");
            }
        }

        cleaned_parts.join(",")
    }

    pub fn count_records(&self) -> Result<usize, Box<dyn Error>> {
        let file = File::open(&self.input_path)?;
        let reader = BufReader::new(file);
        let count = reader.lines().count();
        Ok(count.saturating_sub(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn test_data_cleaner() {
        let test_input = "name,age,city\nJohn,25,New York\nJane,,London\nBob,30,NULL";
        let input_path = "test_input.csv";
        let output_path = "test_output.csv";

        let mut input_file = File::create(input_path).unwrap();
        input_file.write_all(test_input.as_bytes()).unwrap();

        let cleaner = DataCleaner::new(input_path, output_path);
        cleaner.clean_csv().unwrap();

        let mut output_file = File::open(output_path).unwrap();
        let mut content = String::new();
        output_file.read_to_string(&mut content).unwrap();

        assert!(content.contains("John,25,New York"));
        assert!(content.contains("Jane,,London"));
        assert!(content.contains("Bob,30,"));

        std::fs::remove_file(input_path).unwrap();
        std::fs::remove_file(output_path).unwrap();
    }
}