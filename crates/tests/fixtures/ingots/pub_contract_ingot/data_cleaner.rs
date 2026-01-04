use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

pub fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut output_file = File::create(output_path)?;
    
    let mut headers: Vec<String> = Vec::new();
    let mut data_rows: Vec<HashMap<String, String>> = Vec::new();
    
    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        
        if line_num == 0 {
            headers = line.split(',')
                .map(|h| h.trim().to_lowercase().replace(' ', "_"))
                .collect();
            writeln!(output_file, "{}", headers.join(","))?;
            continue;
        }
        
        let values: Vec<&str> = line.split(',').collect();
        if values.len() != headers.len() {
            continue;
        }
        
        let mut row = HashMap::new();
        for (i, header) in headers.iter().enumerate() {
            let cleaned_value = values[i]
                .trim()
                .replace('\n', " ")
                .replace('\r', "")
                .replace('\"', "'");
            
            row.insert(header.clone(), cleaned_value);
        }
        data_rows.push(row);
    }
    
    for row in data_rows {
        let row_values: Vec<String> = headers.iter()
            .map(|h| row.get(h).unwrap_or(&"".to_string()).to_string())
            .collect();
        writeln!(output_file, "{}", row_values.join(","))?;
    }
    
    Ok(())
}

pub fn validate_email(email: &str) -> bool {
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return false;
    }
    
    let domain_parts: Vec<&str> = parts[1].split('.').collect();
    domain_parts.len() >= 2 && 
    !parts[0].is_empty() && 
    !domain_parts.iter().any(|p| p.is_empty())
}