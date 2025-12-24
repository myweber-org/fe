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
    
    pub fn column_stats(&self, column_index: usize) -> Option<ColumnStats> {
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
        
        let variance: f64 = numeric_values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count as f64;
        
        let std_dev = variance.sqrt();
        
        Some(ColumnStats {
            column_name: self.headers[column_index].clone(),
            count,
            mean,
            std_dev,
            min: numeric_values.iter().cloned().fold(f64::INFINITY, f64::min),
            max: numeric_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        })
    }
    
    pub fn get_headers(&self) -> &[String] {
        &self.headers
    }
}

pub struct ColumnStats {
    pub column_name: String,
    pub count: usize,
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
}

impl std::fmt::Display for ColumnStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Column: {}\nCount: {}\nMean: {:.4}\nStd Dev: {:.4}\nMin: {:.4}\nMax: {:.4}",
               self.column_name, self.count, self.mean, self.std_dev, self.min, self.max)
    }
}