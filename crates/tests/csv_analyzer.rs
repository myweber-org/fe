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
    
    pub fn column_stats(&self, column_index: usize) -> Result<ColumnStats, Box<dyn Error>> {
        if column_index >= self.headers.len() {
            return Err("Column index out of bounds".into());
        }
        
        let mut numeric_values = Vec::new();
        let mut text_values = Vec::new();
        let mut empty_count = 0;
        
        for row in &self.data {
            if column_index >= row.len() {
                empty_count += 1;
                continue;
            }
            
            let value = &row[column_index];
            if value.trim().is_empty() {
                empty_count += 1;
            } else if let Ok(num) = value.parse::<f64>() {
                numeric_values.push(num);
            } else {
                text_values.push(value.clone());
            }
        }
        
        let stats = ColumnStats {
            column_name: self.headers[column_index].clone(),
            total_values: self.data.len(),
            numeric_count: numeric_values.len(),
            text_count: text_values.len(),
            empty_count,
            numeric_stats: if !numeric_values.is_empty() {
                Some(NumericStats {
                    min: numeric_values.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
                    max: numeric_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)),
                    sum: numeric_values.iter().sum(),
                    avg: numeric_values.iter().sum::<f64>() / numeric_values.len() as f64,
                })
            } else {
                None
            },
            unique_text_count: if !text_values.is_empty() {
                let unique: std::collections::HashSet<_> = text_values.iter().collect();
                Some(unique.len())
            } else {
                None
            },
        };
        
        Ok(stats)
    }
    
    pub fn validate_data(&self) -> Vec<DataIssue> {
        let mut issues = Vec::new();
        
        for (row_idx, row) in self.data.iter().enumerate() {
            if row.len() != self.headers.len() {
                issues.push(DataIssue::ColumnMismatch {
                    row: row_idx + 1,
                    expected: self.headers.len(),
                    actual: row.len(),
                });
            }
            
            for (col_idx, cell) in row.iter().enumerate() {
                if cell.trim().is_empty() {
                    issues.push(DataIssue::EmptyCell {
                        row: row_idx + 1,
                        column: col_idx + 1,
                        column_name: self.headers[col_idx].clone(),
                    });
                }
            }
        }
        
        issues
    }
}

pub struct ColumnStats {
    column_name: String,
    total_values: usize,
    numeric_count: usize,
    text_count: usize,
    empty_count: usize,
    numeric_stats: Option<NumericStats>,
    unique_text_count: Option<usize>,
}

pub struct NumericStats {
    min: f64,
    max: f64,
    sum: f64,
    avg: f64,
}

pub enum DataIssue {
    ColumnMismatch {
        row: usize,
        expected: usize,
        actual: usize,
    },
    EmptyCell {
        row: usize,
        column: usize,
        column_name: String,
    },
}

impl std::fmt::Display for ColumnStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Statistics for column: {}", self.column_name)?;
        writeln!(f, "  Total values: {}", self.total_values)?;
        writeln!(f, "  Numeric values: {}", self.numeric_count)?;
        writeln!(f, "  Text values: {}", self.text_count)?;
        writeln!(f, "  Empty values: {}", self.empty_count)?;
        
        if let Some(stats) = &self.numeric_stats {
            writeln!(f, "  Numeric statistics:")?;
            writeln!(f, "    Min: {:.2}", stats.min)?;
            writeln!(f, "    Max: {:.2}", stats.max)?;
            writeln!(f, "    Sum: {:.2}", stats.sum)?;
            writeln!(f, "    Avg: {:.2}", stats.avg)?;
        }
        
        if let Some(unique_count) = self.unique_text_count {
            writeln!(f, "  Unique text values: {}", unique_count)?;
        }
        
        Ok(())
    }
}

impl std::fmt::Display for DataIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataIssue::ColumnMismatch { row, expected, actual } => {
                write!(f, "Row {}: Expected {} columns, found {}", row, expected, actual)
            }
            DataIssue::EmptyCell { row, column, column_name } => {
                write!(f, "Row {}, Column {} ({}): Empty cell", row, column, column_name)
            }
        }
    }
}