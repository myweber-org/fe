use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct CsvStats {
    pub row_count: usize,
    pub column_count: usize,
    pub numeric_columns: Vec<usize>,
}

pub fn analyze_csv(file_path: &str) -> Result<CsvStats, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    
    let header = match lines.next() {
        Some(Ok(line)) => line,
        _ => return Err("Empty CSV file".into()),
    };
    
    let column_count = header.split(',').count();
    let mut row_count = 1;
    let mut numeric_column_flags = vec![true; column_count];
    
    for line_result in lines {
        let line = line_result?;
        row_count += 1;
        
        let values: Vec<&str> = line.split(',').collect();
        if values.len() != column_count {
            return Err("Inconsistent column count".into());
        }
        
        for (i, value) in values.iter().enumerate() {
            if numeric_column_flags[i] && value.trim().parse::<f64>().is_err() {
                numeric_column_flags[i] = false;
            }
        }
    }
    
    let numeric_columns: Vec<usize> = numeric_column_flags
        .iter()
        .enumerate()
        .filter(|(_, &is_numeric)| is_numeric)
        .map(|(idx, _)| idx)
        .collect();
    
    Ok(CsvStats {
        row_count,
        column_count,
        numeric_columns,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_analyze_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,age,salary").unwrap();
        writeln!(temp_file, "1,Alice,30,50000").unwrap();
        writeln!(temp_file, "2,Bob,25,45000").unwrap();
        
        let stats = analyze_csv(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(stats.row_count, 3);
        assert_eq!(stats.column_count, 4);
        assert_eq!(stats.numeric_columns, vec![0, 2, 3]);
    }
}