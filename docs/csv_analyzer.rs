use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct CsvAnalyzer {
    headers: Vec<String>,
    data: Vec<Vec<String>>,
}

impl CsvAnalyzer {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        
        let headers = if let Some(first_line) = lines.next() {
            first_line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        } else {
            return Err("Empty CSV file".into());
        };
        
        let mut data = Vec::new();
        for line in lines {
            let record: Vec<String> = line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            if record.len() == headers.len() {
                data.push(record);
            }
        }
        
        Ok(CsvAnalyzer { headers, data })
    }
    
    pub fn column_stats(&self, column_index: usize) -> Option<HashMap<String, usize>> {
        if column_index >= self.headers.len() {
            return None;
        }
        
        let mut stats = HashMap::new();
        for record in &self.data {
            let value = &record[column_index];
            *stats.entry(value.clone()).or_insert(0) += 1;
        }
        
        Some(stats)
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
    
    pub fn row_count(&self) -> usize {
        self.data.len()
    }
    
    pub fn column_count(&self) -> usize {
        self.headers.len()
    }
    
    pub fn get_headers(&self) -> &[String] {
        &self.headers
    }
}

pub fn calculate_average(numbers: &[f64]) -> Option<f64> {
    if numbers.is_empty() {
        return None;
    }
    let sum: f64 = numbers.iter().sum();
    Some(sum / numbers.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_csv_analysis() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        writeln!(temp_file, "Charlie,30,Paris").unwrap();
        
        let analyzer = CsvAnalyzer::from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(analyzer.row_count(), 3);
        assert_eq!(analyzer.column_count(), 3);
        
        let age_stats = analyzer.column_stats(1).unwrap();
        assert_eq!(age_stats.get("30"), Some(&2));
        assert_eq!(age_stats.get("25"), Some(&1));
    }
    
    #[test]
    fn test_average_calculation() {
        let numbers = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(calculate_average(&numbers), Some(3.0));
        assert_eq!(calculate_average(&[]), None);
    }
}