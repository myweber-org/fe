use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

pub struct CsvFilter {
    pub column_index: usize,
    pub filter_value: String,
}

impl CsvFilter {
    pub fn new(column_index: usize, filter_value: &str) -> Self {
        CsvFilter {
            column_index,
            filter_value: filter_value.to_string(),
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut filtered_rows = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let columns: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            
            if columns.len() > self.column_index && columns[self.column_index] == self.filter_value {
                filtered_rows.push(columns);
            }
        }

        Ok(filtered_rows)
    }

    pub fn count_matches<P: AsRef<Path>>(&self, file_path: P) -> Result<usize, Box<dyn Error>> {
        let matches = self.process_file(file_path)?;
        Ok(matches.len())
    }
}

pub fn read_csv_headers<P: AsRef<Path>>(file_path: P) -> Result<Vec<String>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);
    let mut header_line = String::new();
    reader.read_line(&mut header_line)?;
    
    let headers: Vec<String> = header_line.trim().split(',').map(|s| s.trim().to_string()).collect();
    Ok(headers)
}

pub fn write_filtered_csv<P: AsRef<Path>>(
    filter: &CsvFilter,
    input_path: P,
    output_path: P,
) -> Result<(), Box<dyn Error>> {
    use std::io::Write;
    
    let filtered_data = filter.process_file(input_path)?;
    let mut output_file = File::create(output_path)?;
    
    for row in filtered_data {
        let line = row.join(",");
        writeln!(output_file, "{}", line)?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_filter() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        writeln!(temp_file, "Charlie,30,Paris").unwrap();
        
        let filter = CsvFilter::new(1, "30");
        let result = filter.process_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0][0], "Alice");
        assert_eq!(result[1][0], "Charlie");
    }

    #[test]
    fn test_count_matches() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,status").unwrap();
        writeln!(temp_file, "1,active").unwrap();
        writeln!(temp_file, "2,inactive").unwrap();
        writeln!(temp_file, "3,active").unwrap();
        
        let filter = CsvFilter::new(1, "active");
        let count = filter.count_matches(temp_file.path()).unwrap();
        
        assert_eq!(count, 2);
    }
}