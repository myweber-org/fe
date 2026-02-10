use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

#[derive(Debug)]
pub struct CsvConfig {
    pub input_path: String,
    pub output_path: String,
    pub filter_column: usize,
    pub filter_value: String,
}

pub fn process_csv(config: &CsvConfig) -> Result<usize, Box<dyn Error>> {
    let input_file = File::open(&config.input_path)?;
    let reader = BufReader::new(input_file);
    
    let output_path = Path::new(&config.output_path);
    let mut output_file = File::create(output_path)?;
    
    let mut lines_processed = 0;
    
    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result?;
        
        if line_num == 0 {
            writeln!(output_file, "{}", line)?;
            continue;
        }
        
        let columns: Vec<&str> = line.split(',').collect();
        
        if columns.len() <= config.filter_column {
            continue;
        }
        
        if columns[config.filter_column] == config.filter_value {
            writeln!(output_file, "{}", line)?;
            lines_processed += 1;
        }
    }
    
    Ok(lines_processed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    
    #[test]
    fn test_csv_filtering() {
        let test_input = "id,name,status\n1,Alice,active\n2,Bob,inactive\n3,Charlie,active";
        let temp_input = "test_input.csv";
        let temp_output = "test_output.csv";
        
        let mut input_file = File::create(temp_input).unwrap();
        write!(input_file, "{}", test_input).unwrap();
        
        let config = CsvConfig {
            input_path: temp_input.to_string(),
            output_path: temp_output.to_string(),
            filter_column: 2,
            filter_value: "active".to_string(),
        };
        
        let result = process_csv(&config);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
        
        let mut output_content = String::new();
        File::open(temp_output)
            .unwrap()
            .read_to_string(&mut output_content)
            .unwrap();
        
        assert!(output_content.contains("Alice"));
        assert!(output_content.contains("Charlie"));
        assert!(!output_content.contains("Bob"));
        
        std::fs::remove_file(temp_input).unwrap();
        std::fs::remove_file(temp_output).unwrap();
    }
}