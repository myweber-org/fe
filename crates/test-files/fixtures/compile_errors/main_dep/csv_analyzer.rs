use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct CsvAnalyzer {
    headers: Vec<String>,
    records: Vec<HashMap<String, String>>,
}

impl CsvAnalyzer {
    pub fn new(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let headers_line = lines.next()
            .ok_or("Empty CSV file")??;
        let headers: Vec<String> = headers_line.split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let mut records = Vec::new();
        for line_result in lines {
            let line = line_result?;
            let values: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            
            if values.len() != headers.len() {
                continue;
            }

            let mut record = HashMap::new();
            for (i, header) in headers.iter().enumerate() {
                record.insert(header.clone(), values[i].to_string());
            }
            records.push(record);
        }

        Ok(CsvAnalyzer { headers, records })
    }

    pub fn row_count(&self) -> usize {
        self.records.len()
    }

    pub fn column_count(&self) -> usize {
        self.headers.len()
    }

    pub fn unique_values(&self, column_name: &str) -> Result<Vec<String>, Box<dyn Error>> {
        let mut values = Vec::new();
        for record in &self.records {
            if let Some(value) = record.get(column_name) {
                values.push(value.clone());
            }
        }
        values.sort();
        values.dedup();
        Ok(values)
    }

    pub fn filter_by_column(&self, column_name: &str, target_value: &str) -> Vec<HashMap<String, String>> {
        self.records.iter()
            .filter(|record| record.get(column_name).map_or(false, |v| v == target_value))
            .cloned()
            .collect()
    }

    pub fn column_summary(&self, column_name: &str) -> Result<HashMap<String, usize>, Box<dyn Error>> {
        let mut summary = HashMap::new();
        for record in &self.records {
            if let Some(value) = record.get(column_name) {
                *summary.entry(value.clone()).or_insert(0) += 1;
            }
        }
        Ok(summary)
    }

    pub fn get_headers(&self) -> &Vec<String> {
        &self.headers
    }

    pub fn sample_records(&self, count: usize) -> Vec<HashMap<String, String>> {
        self.records.iter()
            .take(count)
            .cloned()
            .collect()
    }
}

pub fn analyze_csv(file_path: &str) -> Result<(), Box<dyn Error>> {
    let analyzer = CsvAnalyzer::new(file_path)?;
    
    println!("CSV Analysis Report");
    println!("===================");
    println!("Total rows: {}", analyzer.row_count());
    println!("Total columns: {}", analyzer.column_count());
    println!("\nColumn headers:");
    for header in analyzer.get_headers() {
        println!("  - {}", header);
    }

    if analyzer.row_count() > 0 {
        println!("\nSample records (first 3):");
        for (i, record) in analyzer.sample_records(3).iter().enumerate() {
            println!("Record {}:", i + 1);
            for header in analyzer.get_headers() {
                if let Some(value) = record.get(header) {
                    println!("  {}: {}", header, value);
                }
            }
        }

        if let Some(first_header) = analyzer.get_headers().first() {
            println!("\nUnique values for column '{}':", first_header);
            let unique_vals = analyzer.unique_values(first_header)?;
            for val in unique_vals.iter().take(5) {
                println!("  - {}", val);
            }
            if unique_vals.len() > 5 {
                println!("  ... and {} more", unique_vals.len() - 5);
            }
        }
    }

    Ok(())
}