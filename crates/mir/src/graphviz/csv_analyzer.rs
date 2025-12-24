use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvStats {
    pub row_count: usize,
    pub column_count: usize,
    pub has_header: bool,
    pub sample_rows: Vec<Vec<String>>,
}

pub fn analyze_csv<P: AsRef<Path>>(file_path: P, sample_size: usize) -> Result<CsvStats, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let first_line = match lines.next() {
        Some(Ok(line)) => line,
        Some(Err(e)) => return Err(Box::new(e)),
        None => return Err("Empty file".into()),
    };

    let columns: Vec<String> = first_line.split(',').map(|s| s.trim().to_string()).collect();
    let column_count = columns.len();
    
    let mut row_count = 1;
    let mut sample_rows = Vec::with_capacity(sample_size);
    sample_rows.push(columns.clone());

    let mut has_header = true;
    for column in &columns {
        if column.parse::<f64>().is_ok() {
            has_header = false;
            break;
        }
    }

    for line in lines.take(sample_size - 1) {
        let line = line?;
        let row: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
        
        if row.len() != column_count {
            return Err(format!("Row {} has {} columns, expected {}", row_count + 1, row.len(), column_count).into());
        }
        
        sample_rows.push(row);
        row_count += 1;
    }

    for line in lines {
        let _ = line?;
        row_count += 1;
    }

    Ok(CsvStats {
        row_count,
        column_count,
        has_header,
        sample_rows,
    })
}

pub fn validate_csv_format<P: AsRef<Path>>(file_path: P) -> Result<bool, Box<dyn Error>> {
    let stats = analyze_csv(file_path, 10)?;
    
    if stats.row_count == 0 {
        return Ok(false);
    }
    
    if stats.column_count == 0 {
        return Ok(false);
    }
    
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", content).unwrap();
        file
    }

    #[test]
    fn test_valid_csv() {
        let csv = "name,age,city\nAlice,30,New York\nBob,25,London";
        let file = create_test_csv(csv);
        
        let stats = analyze_csv(file.path(), 5).unwrap();
        assert_eq!(stats.row_count, 2);
        assert_eq!(stats.column_count, 3);
        assert!(stats.has_header);
        assert_eq!(stats.sample_rows.len(), 3);
    }

    #[test]
    fn test_csv_without_header() {
        let csv = "Alice,30,New York\nBob,25,London\nCharlie,35,Paris";
        let file = create_test_csv(csv);
        
        let stats = analyze_csv(file.path(), 5).unwrap();
        assert_eq!(stats.row_count, 3);
        assert_eq!(stats.column_count, 3);
        assert!(!stats.has_header);
    }

    #[test]
    fn test_invalid_column_count() {
        let csv = "name,age,city\nAlice,30\nBob,25,London,extra";
        let file = create_test_csv(csv);
        
        let result = analyze_csv(file.path(), 5);
        assert!(result.is_err());
    }
}