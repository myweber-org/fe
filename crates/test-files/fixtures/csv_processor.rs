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
            
            if line_num == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }
            
            let columns: Vec<&str> = line.split(',').collect();
            
            if columns.get(self.filter_column)
                .map(|&val| val.trim() == self.filter_value)
                .unwrap_or(false)
            {
                let transformed_line = self.transform_line(&columns);
                writeln!(output_file, "{}", transformed_line)?;
                processed_count += 1;
            }
        }
        
        Ok(processed_count)
    }
    
    fn transform_line(&self, columns: &[&str]) -> String {
        let mut transformed: Vec<String> = columns
            .iter()
            .map(|&col| col.trim().to_uppercase())
            .collect();
        
        if transformed.len() > 1 {
            transformed.swap(0, 1);
        }
        
        transformed.join(",")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    
    #[test]
    fn test_csv_processing() {
        let test_input = "id,name,value\n1,test,active\n2,demo,inactive\n3,test,active";
        let temp_input = "test_input.csv";
        let temp_output = "test_output.csv";
        
        std::fs::write(temp_input, test_input).unwrap();
        
        let processor = CsvProcessor::new(temp_input, temp_output, 1, "test");
        let result = processor.process();
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
        
        let mut output_content = String::new();
        File::open(temp_output)
            .unwrap()
            .read_to_string(&mut output_content)
            .unwrap();
        
        assert!(output_content.contains("TEST,1,ACTIVE"));
        assert!(output_content.contains("TEST,3,ACTIVE"));
        
        std::fs::remove_file(temp_input).unwrap();
        std::fs::remove_file(temp_output).unwrap();
    }
}