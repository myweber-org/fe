use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvStats {
    pub row_count: usize,
    pub column_count: usize,
    pub has_header: bool,
}

pub fn analyze_csv(file_path: &str) -> Result<CsvStats, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    
    let first_line = match lines.next() {
        Some(line) => line?,
        None => return Err("Empty file".into()),
    };
    
    let column_count = first_line.split(',').count();
    let mut row_count = 1;
    
    for line in lines {
        let _ = line?;
        row_count += 1;
    }
    
    let has_header = first_line.chars().any(|c| c.is_alphabetic());
    
    Ok(CsvStats {
        row_count,
        column_count,
        has_header,
    })
}

pub fn print_stats(stats: &CsvStats) {
    println!("CSV Analysis Results:");
    println!("  Rows: {}", stats.row_count);
    println!("  Columns: {}", stats.column_count);
    println!("  Has header: {}", stats.has_header);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_analyze_csv_with_header() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        
        let stats = analyze_csv(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(stats.row_count, 3);
        assert_eq!(stats.column_count, 3);
        assert!(stats.has_header);
    }
    
    #[test]
    fn test_analyze_csv_without_header() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,2,3").unwrap();
        writeln!(temp_file, "4,5,6").unwrap();
        
        let stats = analyze_csv(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(stats.row_count, 2);
        assert_eq!(stats.column_count, 3);
        assert!(!stats.has_header);
    }
}