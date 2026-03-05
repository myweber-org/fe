
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
        Some(Err(e)) => return Err(Box::new(e)),
        None => return Err("Empty input file".into()),
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
    use std::fs;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_remove_duplicates() {
        let input_content = "id,name,value\n1,Alice,100\n2,Bob,200\n1,Alice,100\n3,Charlie,300\n2,Bob,200";
        let expected_output = "id,name,value\n1,Alice,100\n2,Bob,200\n3,Charlie,300";
        
        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        
        fs::write(input_file.path(), input_content).unwrap();
        
        remove_duplicates(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap()
        ).unwrap();
        
        let actual_output = fs::read_to_string(output_file.path()).unwrap();
        assert_eq!(actual_output.trim(), expected_output);
    }
}