use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::collections::HashMap;

pub struct CsvProcessor {
    headers: Vec<String>,
    data: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn new(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
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
            let line = line?;
            let row: Vec<String> = line.split(',')
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
        self.data.iter()
            .filter(|row| predicate(row))
            .cloned()
            .collect()
    }

    pub fn aggregate_by_column(&self, column_index: usize, operation: &str) -> Option<f64> {
        if column_index >= self.headers.len() {
            return None;
        }

        let values: Vec<f64> = self.data.iter()
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

    pub fn count_by_column(&self, column_index: usize) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        
        for row in &self.data {
            if column_index < row.len() {
                let key = &row[column_index];
                *counts.entry(key.clone()).or_insert(0) += 1;
            }
        }
        
        counts
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

pub fn process_csv_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let processor = CsvProcessor::new(input_path)?;
    
    println!("Processing CSV with {} rows and {} columns", 
             processor.row_count(), 
             processor.column_count());
    
    println!("Headers: {:?}", processor.get_headers());
    
    if let Some(avg) = processor.aggregate_by_column(2, "avg") {
        println!("Average of column 2: {:.2}", avg);
    }
    
    let filtered = processor.filter_rows(|row| {
        row.len() > 1 && row[1].contains("active")
    });
    
    println!("Filtered rows count: {}", filtered.len());
    
    let counts = processor.count_by_column(0);
    println!("Unique values in first column: {}", counts.len());
    
    let output_file = File::create(output_path)?;
    let mut writer = csv::Writer::from_writer(output_file);
    
    writer.write_record(processor.get_headers())?;
    
    for row in filtered {
        writer.write_record(&row)?;
    }
    
    writer.flush()?;
    println!("Results written to: {}", output_path);
    
    Ok(())
}use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

fn load_csv_data(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        records.push(record);
    }

    Ok(records)
}

fn filter_active_records(records: &[Record]) -> Vec<&Record> {
    records.iter().filter(|r| r.active).collect()
}

fn calculate_category_averages(records: &[Record]) -> Vec<(String, f64)> {
    use std::collections::HashMap;

    let mut category_sums: HashMap<String, (f64, usize)> = HashMap::new();

    for record in records {
        let entry = category_sums
            .entry(record.category.clone())
            .or_insert((0.0, 0));
        entry.0 += record.value;
        entry.1 += 1;
    }

    category_sums
        .into_iter()
        .map(|(category, (sum, count))| (category, sum / count as f64))
        .collect()
}

fn save_processed_data(
    records: &[&Record],
    averages: &[(String, f64)],
    output_path: &str,
) -> Result<(), Box<dyn Error>> {
    let mut writer = Writer::from_path(output_path)?;

    writer.write_record(&["ID", "Name", "Category", "Value", "Status"])?;
    for record in records {
        writer.serialize((
            record.id,
            &record.name,
            &record.category,
            record.value,
            "Active",
        ))?;
    }

    writer.write_record(&[])?;
    writer.write_record(&["Category", "Average Value"])?;
    for (category, avg) in averages {
        writer.serialize((category, avg))?;
    }

    writer.flush()?;
    Ok(())
}

fn process_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let records = load_csv_data(input_path)?;
    let active_records = filter_active_records(&records);
    let category_averages = calculate_category_averages(&records);

    println!("Total records loaded: {}", records.len());
    println!("Active records found: {}", active_records.len());
    println!("Categories processed: {}", category_averages.len());

    save_processed_data(&active_records, &category_averages, output_path)?;
    println!("Processed data saved to: {}", output_path);

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/processed_output.csv";

    match process_csv_data(input_file, output_file) {
        Ok(_) => println!("CSV processing completed successfully"),
        Err(e) => eprintln!("Error processing CSV data: {}", e),
    }

    Ok(())
}