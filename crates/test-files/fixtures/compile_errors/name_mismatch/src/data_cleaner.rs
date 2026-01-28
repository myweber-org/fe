
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub fn remove_duplicates(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let path = Path::new(input_path);
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    
    let mut unique_lines = HashSet::new();
    let mut lines = Vec::new();
    
    for line_result in reader.lines() {
        let line = line_result?;
        if unique_lines.insert(line.clone()) {
            lines.push(line);
        }
    }
    
    let mut output_file = File::create(output_path)?;
    for line in lines {
        writeln!(output_file, "{}", line)?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[test]
    fn test_remove_duplicates() {
        let test_input = "test_input.csv";
        let test_output = "test_output.csv";
        
        let test_data = "id,name,value\n1,Alice,100\n2,Bob,200\n1,Alice,100\n3,Charlie,300\n2,Bob,200";
        fs::write(test_input, test_data).unwrap();
        
        remove_duplicates(test_input, test_output).unwrap();
        
        let result = fs::read_to_string(test_output).unwrap();
        let expected = "id,name,value\n1,Alice,100\n2,Bob,200\n3,Charlie,300\n";
        
        assert_eq!(result, expected);
        
        fs::remove_file(test_input).unwrap();
        fs::remove_file(test_output).unwrap();
    }
}