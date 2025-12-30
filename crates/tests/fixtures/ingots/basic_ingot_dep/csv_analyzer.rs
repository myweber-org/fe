use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvAnalyzer {
    file_path: String,
    delimiter: char,
    has_header: bool,
}

impl CsvAnalyzer {
    pub fn new(file_path: &str) -> Self {
        CsvAnalyzer {
            file_path: file_path.to_string(),
            delimiter: ',',
            has_header: true,
        }
    }

    pub fn with_delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = delimiter;
        self
    }

    pub fn with_header(mut self, has_header: bool) -> Self {
        self.has_header = has_header;
        self
    }

    pub fn analyze(&self) -> Result<AnalysisResult, Box<dyn Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let mut row_count = 0;
        let mut column_count = 0;
        let mut empty_cells = 0;
        let mut numeric_cells = 0;
        let mut text_cells = 0;

        let mut headers = Vec::new();
        let mut column_stats: Vec<ColumnStat> = Vec::new();

        for (line_num, line) in lines.enumerate() {
            let line = line?;
            
            if line_num == 0 && self.has_header {
                headers = line.split(self.delimiter)
                    .map(|s| s.trim().to_string())
                    .collect();
                column_count = headers.len();
                column_stats = vec![ColumnStat::new(); column_count];
                continue;
            }

            let fields: Vec<&str> = line.split(self.delimiter).collect();
            
            if column_count == 0 {
                column_count = fields.len();
                column_stats = vec![ColumnStat::new(); column_count];
            }

            if fields.len() != column_count {
                return Err(format!("Row {} has {} columns, expected {}", 
                    row_count + 1, fields.len(), column_count).into());
            }

            row_count += 1;

            for (i, field) in fields.iter().enumerate() {
                let trimmed = field.trim();
                
                if trimmed.is_empty() {
                    empty_cells += 1;
                    column_stats[i].empty_count += 1;
                } else if trimmed.parse::<f64>().is_ok() {
                    numeric_cells += 1;
                    column_stats[i].numeric_count += 1;
                    
                    if let Ok(num) = trimmed.parse::<f64>() {
                        column_stats[i].update_numeric_stats(num);
                    }
                } else {
                    text_cells += 1;
                    column_stats[i].text_count += 1;
                    
                    if trimmed.len() > column_stats[i].max_text_length {
                        column_stats[i].max_text_length = trimmed.len();
                    }
                }
            }
        }

        Ok(AnalysisResult {
            file_path: self.file_path.clone(),
            row_count,
            column_count,
            empty_cells,
            numeric_cells,
            text_cells,
            headers,
            column_stats,
        })
    }
}

#[derive(Debug, Clone)]
struct ColumnStat {
    numeric_count: usize,
    text_count: usize,
    empty_count: usize,
    min_value: Option<f64>,
    max_value: Option<f64>,
    sum: f64,
    max_text_length: usize,
}

impl ColumnStat {
    fn new() -> Self {
        ColumnStat {
            numeric_count: 0,
            text_count: 0,
            empty_count: 0,
            min_value: None,
            max_value: None,
            sum: 0.0,
            max_text_length: 0,
        }
    }

    fn update_numeric_stats(&mut self, value: f64) {
        self.sum += value;
        
        match self.min_value {
            Some(min) if value < min => self.min_value = Some(value),
            None => self.min_value = Some(value),
            _ => {}
        }
        
        match self.max_value {
            Some(max) if value > max => self.max_value = Some(value),
            None => self.max_value = Some(value),
            _ => {}
        }
    }
}

#[derive(Debug)]
pub struct AnalysisResult {
    file_path: String,
    row_count: usize,
    column_count: usize,
    empty_cells: usize,
    numeric_cells: usize,
    text_cells: usize,
    headers: Vec<String>,
    column_stats: Vec<ColumnStat>,
}

impl AnalysisResult {
    pub fn print_summary(&self) {
        println!("CSV Analysis Summary");
        println!("====================");
        println!("File: {}", self.file_path);
        println!("Rows: {}", self.row_count);
        println!("Columns: {}", self.column_count);
        println!("Total cells: {}", self.row_count * self.column_count);
        println!("Empty cells: {} ({:.1}%)", 
            self.empty_cells,
            (self.empty_cells as f64 / (self.row_count * self.column_count) as f64) * 100.0);
        println!("Numeric cells: {}", self.numeric_cells);
        println!("Text cells: {}", self.text_cells);
        
        if !self.headers.is_empty() {
            println!("\nColumn Details:");
            for (i, header) in self.headers.iter().enumerate() {
                let stat = &self.column_stats[i];
                println!("  Column {}: '{}'", i + 1, header);
                println!("    Numeric values: {}", stat.numeric_count);
                println!("    Text values: {}", stat.text_count);
                println!("    Empty values: {}", stat.empty_count);
                
                if stat.numeric_count > 0 {
                    println!("    Numeric range: [{:.2}, {:.2}]", 
                        stat.min_value.unwrap_or(0.0),
                        stat.max_value.unwrap_or(0.0));
                    println!("    Average: {:.2}", stat.sum / stat.numeric_count as f64);
                }
                
                if stat.max_text_length > 0 {
                    println!("    Max text length: {}", stat.max_text_length);
                }
            }
        }
    }
}

pub fn analyze_csv_file(file_path: &str) -> Result<(), Box<dyn Error>> {
    let analyzer = CsvAnalyzer::new(file_path);
    let result = analyzer.analyze()?;
    result.print_summary();
    Ok(())
}