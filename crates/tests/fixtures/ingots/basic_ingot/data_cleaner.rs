
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub fn remove_duplicates(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(Path::new(input_path))?;
    let reader = BufReader::new(input_file);
    let mut lines = reader.lines();
    
    let header = match lines.next() {
        Some(Ok(h)) => h,
        _ => return Err("Empty input file".into()),
    };
    
    let mut seen = HashSet::new();
    let mut unique_lines = Vec::new();
    
    for line_result in lines {
        let line = line_result?;
        if !seen.contains(&line) {
            seen.insert(line.clone());
            unique_lines.push(line);
        }
    }
    
    let mut output_file = File::create(Path::new(output_path))?;
    writeln!(output_file, "{}", header)?;
    
    for line in unique_lines {
        writeln!(output_file, "{}", line)?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    
    #[test]
    fn test_remove_duplicates() {
        let test_input = "id,name,value\n1,test,100\n2,test,200\n1,test,100\n3,other,300";
        let input_path = "test_input.csv";
        let output_path = "test_output.csv";
        
        std::fs::write(input_path, test_input).unwrap();
        
        remove_duplicates(input_path, output_path).unwrap();
        
        let mut output_content = String::new();
        File::open(output_path)
            .unwrap()
            .read_to_string(&mut output_content)
            .unwrap();
            
        let expected = "id,name,value\n1,test,100\n2,test,200\n3,other,300\n";
        assert_eq!(output_content, expected);
        
        std::fs::remove_file(input_path).unwrap();
        std::fs::remove_file(output_path).unwrap();
    }
}