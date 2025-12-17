use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub fn merge_csv_files(input_paths: &[&str], output_path: &str, deduplicate: bool, sort_by_column: Option<usize>) -> Result<(), Box<dyn Error>> {
    let mut all_records = Vec::new();
    let mut headers = None;
    
    for input_path in input_paths {
        let file = File::open(input_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        
        if let Some(header_line) = lines.next() {
            let current_headers = header_line?;
            
            if headers.is_none() {
                headers = Some(current_headers.clone());
            } else if headers.as_ref() != Some(&current_headers) {
                return Err("CSV files have different headers".into());
            }
            
            for line in lines {
                let record = line?;
                if !record.trim().is_empty() {
                    all_records.push(record);
                }
            }
        }
    }
    
    if deduplicate {
        let unique_records: HashSet<String> = all_records.drain(..).collect();
        all_records = unique_records.into_iter().collect();
    }
    
    if let Some(column_index) = sort_by_column {
        all_records.sort_by(|a, b| {
            let a_fields: Vec<&str> = a.split(',').collect();
            let b_fields: Vec<&str> = b.split(',').collect();
            
            let a_value = a_fields.get(column_index).unwrap_or(&"");
            let b_value = b_fields.get(column_index).unwrap_or(&"");
            
            a_value.cmp(b_value)
        });
    }
    
    let mut output_file = File::create(output_path)?;
    
    if let Some(ref headers) = headers {
        writeln!(output_file, "{}", headers)?;
    }
    
    for record in all_records {
        writeln!(output_file, "{}", record)?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_merge_csv_files() {
        let csv1 = "id,name,value\n1,Alice,100\n2,Bob,200";
        let csv2 = "id,name,value\n3,Charlie,300\n4,Diana,400";
        
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        
        fs::write(file1.path(), csv1).unwrap();
        fs::write(file2.path(), csv2).unwrap();
        
        let input_paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];
        
        merge_csv_files(&input_paths, output_file.path().to_str().unwrap(), false, None).unwrap();
        
        let result = fs::read_to_string(output_file.path()).unwrap();
        let expected = "id,name,value\n1,Alice,100\n2,Bob,200\n3,Charlie,300\n4,Diana,400\n";
        
        assert_eq!(result, expected);
    }
    
    #[test]
    fn test_deduplicate() {
        let csv1 = "id,name,value\n1,Alice,100\n2,Bob,200";
        let csv2 = "id,name,value\n2,Bob,200\n3,Charlie,300";
        
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        
        fs::write(file1.path(), csv1).unwrap();
        fs::write(file2.path(), csv2).unwrap();
        
        let input_paths = [
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap(),
        ];
        
        merge_csv_files(&input_paths, output_file.path().to_str().unwrap(), true, None).unwrap();
        
        let result = fs::read_to_string(output_file.path()).unwrap();
        let lines: Vec<&str> = result.lines().collect();
        
        assert_eq!(lines.len(), 4);
        assert!(lines.contains(&"1,Alice,100"));
        assert!(lines.contains(&"2,Bob,200"));
        assert!(lines.contains(&"3,Charlie,300"));
    }
}