
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    headers: Vec<String>,
    data: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn new(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        
        let headers = match lines.next() {
            Some(header_line) => header_line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            None => return Err("Empty CSV file".into()),
        };
        
        let mut data = Vec::new();
        for line in lines {
            let row: Vec<String> = line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            if row.len() == headers.len() {
                data.push(row);
            }
        }
        
        Ok(CsvProcessor { headers, data })
    }
    
    pub fn filter_rows<F>(&self, predicate: F) -> Vec<Vec<String>>
    where
        F: Fn(&[String]) -> bool,
    {
        self.data
            .iter()
            .filter(|row| predicate(row))
            .cloned()
            .collect()
    }
    
    pub fn aggregate_column(&self, column_index: usize, operation: &str) -> Option<f64> {
        if column_index >= self.headers.len() {
            return None;
        }
        
        let values: Vec<f64> = self.data
            .iter()
            .filter_map(|row| row[column_index].parse::<f64>().ok())
            .collect();
        
        if values.is_empty() {
            return None;
        }
        
        match operation {
            "sum" => Some(values.iter().sum()),
            "avg" => Some(values.iter().sum::<f64>() / values.len() as f64),
            "max" => values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).copied(),
            "min" => values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).copied(),
            _ => None,
        }
    }
    
    pub fn get_headers(&self) -> &[String] {
        &self.headers
    }
    
    pub fn row_count(&self) -> usize {
        self.data.len()
    }
    
    pub fn column_count(&self) -> usize {
        self.headers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000").unwrap();
        writeln!(temp_file, "Bob,25,45000").unwrap();
        writeln!(temp_file, "Charlie,35,60000").unwrap();
        
        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.row_count(), 3);
        assert_eq!(processor.column_count(), 3);
        
        let filtered = processor.filter_rows(|row| row[1].parse::<i32>().unwrap_or(0) > 30);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0][0], "Charlie");
        
        let avg_salary = processor.aggregate_column(2, "avg").unwrap();
        assert!((avg_salary - 51666.666).abs() < 0.001);
    }
}