
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
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

        let mut records = Vec::new();
        for line in lines {
            let record: Vec<String> = line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            if record.len() == headers.len() {
                records.push(record);
            }
        }

        Ok(CsvProcessor { headers, records })
    }

    pub fn filter_by_column(&self, column_name: &str, predicate: fn(&str) -> bool) -> Vec<Vec<String>> {
        let column_index = match self.headers.iter().position(|h| h == column_name) {
            Some(idx) => idx,
            None => return Vec::new(),
        };

        self.records
            .iter()
            .filter(|record| predicate(&record[column_index]))
            .cloned()
            .collect()
    }

    pub fn aggregate_numeric_column(&self, column_name: &str, operation: &str) -> Result<f64, String> {
        let column_index = match self.headers.iter().position(|h| h == column_name) {
            Some(idx) => idx,
            None => return Err(format!("Column '{}' not found", column_name)),
        };

        let values: Vec<f64> = self.records
            .iter()
            .filter_map(|record| record[column_index].parse().ok())
            .collect();

        if values.is_empty() {
            return Err("No valid numeric values found".into());
        }

        match operation {
            "sum" => Ok(values.iter().sum()),
            "avg" => Ok(values.iter().sum::<f64>() / values.len() as f64),
            "min" => Ok(values.iter().fold(f64::INFINITY, |a, &b| a.min(b))),
            "max" => Ok(values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b))),
            _ => Err(format!("Unknown operation: {}", operation)),
        }
    }

    pub fn write_to_file(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let mut file = File::create(path)?;
        
        writeln!(file, "{}", self.headers.join(","))?;
        
        for record in &self.records {
            writeln!(file, "{}", record.join(","))?;
        }
        
        Ok(())
    }

    pub fn add_column(&mut self, column_name: &str, generator: fn(&[String]) -> String) {
        self.headers.push(column_name.to_string());
        
        for record in &mut self.records {
            let new_value = generator(record);
            record.push(new_value);
        }
    }
}

pub fn process_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let mut processor = CsvProcessor::from_file(input_path)?;
    
    println!("Loaded {} records with columns: {:?}", 
             processor.records.len(), 
             processor.headers);
    
    if processor.headers.contains(&"age".to_string()) {
        let avg_age = processor.aggregate_numeric_column("age", "avg")?;
        println!("Average age: {:.2}", avg_age);
        
        let adults = processor.filter_by_column("age", |age_str| {
            age_str.parse::<u32>().map_or(false, |age| age >= 18)
        });
        println!("Adult records: {}", adults.len());
    }
    
    processor.add_column("category", |record| {
        if record.len() > 1 {
            format!("group_{}", record[0].len() % 3)
        } else {
            "unknown".to_string()
        }
    });
    
    processor.write_to_file(output_path)?;
    println!("Processed data written to: {}", output_path);
    
    Ok(())
}