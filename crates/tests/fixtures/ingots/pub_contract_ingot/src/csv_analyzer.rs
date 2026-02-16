
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct CsvAnalyzer {
    data: Vec<Vec<String>>,
    headers: Vec<String>,
}

impl CsvAnalyzer {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        
        let headers: Vec<String> = rdr.headers()?.iter().map(|s| s.to_string()).collect();
        
        let mut data = Vec::new();
        for result in rdr.records() {
            let record = result?;
            let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
            data.push(row);
        }
        
        Ok(CsvAnalyzer { data, headers })
    }
    
    pub fn row_count(&self) -> usize {
        self.data.len()
    }
    
    pub fn column_count(&self) -> usize {
        self.headers.len()
    }
    
    pub fn get_column_stats(&self, column_index: usize) -> Option<ColumnStats> {
        if column_index >= self.headers.len() {
            return None;
        }
        
        let mut numeric_values = Vec::new();
        for row in &self.data {
            if let Some(value) = row.get(column_index) {
                if let Ok(num) = value.parse::<f64>() {
                    numeric_values.push(num);
                }
            }
        }
        
        if numeric_values.is_empty() {
            return None;
        }
        
        let sum: f64 = numeric_values.iter().sum();
        let count = numeric_values.len();
        let mean = sum / count as f64;
        
        let min = numeric_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = numeric_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        Some(ColumnStats {
            column_name: self.headers[column_index].clone(),
            numeric_count: count,
            total_count: self.data.len(),
            mean,
            min,
            max,
        })
    }
    
    pub fn filter_rows<F>(&self, predicate: F) -> Vec<Vec<String>>
    where
        F: Fn(&[String]) -> bool,
    {
        self.data.iter()
            .filter(|row| predicate(row))
            .cloned()
            .collect()
    }
    
    pub fn get_headers(&self) -> &[String] {
        &self.headers
    }
    
    pub fn sample_data(&self, n: usize) -> Vec<Vec<String>> {
        self.data.iter()
            .take(n)
            .cloned()
            .collect()
    }
}

pub struct ColumnStats {
    pub column_name: String,
    pub numeric_count: usize,
    pub total_count: usize,
    pub mean: f64,
    pub min: f64,
    pub max: f64,
}

impl std::fmt::Display for ColumnStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Column: {}\n", self.column_name)?;
        write!(f, "  Numeric values: {}/{}\n", self.numeric_count, self.total_count)?;
        write!(f, "  Mean: {:.2}\n", self.mean)?;
        write!(f, "  Range: [{:.2}, {:.2}]", self.min, self.max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_csv_analysis() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000").unwrap();
        writeln!(temp_file, "Bob,25,45000").unwrap();
        writeln!(temp_file, "Charlie,35,60000").unwrap();
        
        let analyzer = CsvAnalyzer::new(temp_file.path()).unwrap();
        
        assert_eq!(analyzer.row_count(), 3);
        assert_eq!(analyzer.column_count(), 3);
        
        let age_stats = analyzer.get_column_stats(1).unwrap();
        assert_eq!(age_stats.mean, 30.0);
        assert_eq!(age_stats.min, 25.0);
        assert_eq!(age_stats.max, 35.0);
        
        let high_salary = analyzer.filter_rows(|row| {
            if let Some(salary_str) = row.get(2) {
                if let Ok(salary) = salary_str.parse::<f64>() {
                    return salary > 50000.0;
                }
            }
            false
        });
        
        assert_eq!(high_salary.len(), 1);
        assert_eq!(high_salary[0][0], "Charlie");
    }
}