use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
struct Record {
    id: u32,
    category: String,
    value: f64,
    active: bool,
}

impl Record {
    fn from_csv_line(line: &str) -> Result<Self, Box<dyn Error>> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err("Invalid CSV format".into());
        }

        Ok(Record {
            id: parts[0].parse()?,
            category: parts[1].to_string(),
            value: parts[2].parse()?,
            active: parts[3].parse()?,
        })
    }
}

struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    fn new() -> Self {
        DataProcessor { records: Vec::new() }
    }

    fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines().skip(1) {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let record = Record::from_csv_line(&line)?;
            self.records.push(record);
        }

        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.category == category && r.active)
            .collect()
    }

    fn aggregate_by_category(&self) -> HashMap<String, f64> {
        let mut aggregates = HashMap::new();
        
        for record in &self.records {
            if record.active {
                *aggregates.entry(record.category.clone()).or_insert(0.0) += record.value;
            }
        }
        
        aggregates
    }

    fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }
}

fn process_data_file(input_path: &str) -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    processor.load_from_file(input_path)?;

    println!("Total records loaded: {}", processor.records.len());
    println!("Average value: {:.2}", processor.calculate_average());

    if let Some(max_record) = processor.find_max_value() {
        println!("Maximum value record: {:?}", max_record);
    }

    let aggregates = processor.aggregate_by_category();
    for (category, total) in aggregates {
        println!("Category '{}' total: {:.2}", category, total);
    }

    let filtered = processor.filter_by_category("premium");
    println!("Active premium records: {}", filtered.len());

    Ok(())
}

fn main() {
    let input_file = "data.csv";
    
    match process_data_file(input_file) {
        Ok(_) => println!("Data processing completed successfully"),
        Err(e) => eprintln!("Error processing data: {}", e),
    }
}