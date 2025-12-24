use std::fs::File;
use std::io::{BufRead, BufReader, Write};

pub fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut output_file = File::create(output_path)?;

    for line in reader.lines() {
        let mut line = line?;
        line = line.trim().to_string();
        
        if !line.is_empty() {
            let cleaned_columns: Vec<String> = line
                .split(',')
                .map(|col| col.trim().to_string())
                .collect();
            
            writeln!(output_file, "{}", cleaned_columns.join(","))?;
        }
    }
    
    Ok(())
}