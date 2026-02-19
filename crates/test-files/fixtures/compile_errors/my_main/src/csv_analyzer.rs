
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug)]
pub struct CsvStats {
    pub row_count: usize,
    pub column_count: usize,
    pub has_headers: bool,
    pub sample_data: Vec<Vec<String>>,
}

pub fn analyze_csv(file_path: &str) -> Result<CsvStats, Box<dyn Error>> {
    let path = Path::new(file_path);
    let file = File::open(path)?;
    let mut rdr = csv::Reader::from_reader(file);
    
    let headers = rdr.headers()?.clone();
    let has_headers = !headers.is_empty();
    let column_count = headers.len();
    
    let mut row_count = 0;
    let mut sample_data = Vec::new();
    let sample_size = 5;
    
    for result in rdr.records() {
        let record = result?;
        row_count += 1;
        
        if row_count <= sample_size {
            let row_data: Vec<String> = record.iter().map(|s| s.to_string()).collect();
            sample_data.push(row_data);
        }
    }
    
    Ok(CsvStats {
        row_count,
        column_count,
        has_headers,
        sample_data,
    })
}

pub fn validate_csv_format(file_path: &str) -> Result<bool, Box<dyn Error>> {
    let stats = analyze_csv(file_path)?;
    
    if stats.column_count == 0 {
        return Ok(false);
    }
    
    for row in &stats.sample_data {
        if row.len() != stats.column_count {
            return Ok(false);
        }
    }
    
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_valid_csv_analysis() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        
        let stats = analyze_csv(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(stats.row_count, 2);
        assert_eq!(stats.column_count, 3);
        assert!(stats.has_headers);
        assert_eq!(stats.sample_data.len(), 2);
    }
    
    #[test]
    fn test_csv_validation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "col1,col2,col3").unwrap();
        writeln!(temp_file, "val1,val2,val3").unwrap();
        writeln!(temp_file, "val4,val5,val6").unwrap();
        
        let is_valid = validate_csv_format(temp_file.path().to_str().unwrap()).unwrap();
        assert!(is_valid);
    }
}