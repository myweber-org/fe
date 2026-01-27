
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

impl Record {
    fn from_csv_line(line: &str) -> Result<Self, Box<dyn Error>> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 5 {
            return Err("Invalid CSV format".into());
        }

        Ok(Record {
            id: parts[0].parse()?,
            name: parts[1].to_string(),
            category: parts[2].to_string(),
            value: parts[3].parse()?,
            active: parts[4].parse()?,
        })
    }
}

struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
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
            .filter(|record| record.category == category)
            .collect()
    }

    fn filter_active(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.active)
            .collect()
    }

    fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|record| record.value).sum()
    }

    fn calculate_average_value(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        self.calculate_total_value() / self.records.len() as f64
    }

    fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&Record>> {
        let mut groups = std::collections::HashMap::new();
        
        for record in &self.records {
            groups
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record);
        }
        
        groups
    }
}

fn process_data_sample() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    let sample_data = "id,name,category,value,active\n\
                       1,ItemA,Electronics,250.50,true\n\
                       2,ItemB,Furniture,150.75,true\n\
                       3,ItemC,Electronics,99.99,false\n\
                       4,ItemD,Books,45.25,true\n\
                       5,ItemE,Electronics,300.00,true";
    
    let temp_file = "temp_sample.csv";
    std::fs::write(temp_file, sample_data)?;
    
    processor.load_from_file(temp_file)?;
    std::fs::remove_file(temp_file)?;
    
    let electronics = processor.filter_by_category("Electronics");
    println!("Electronics items: {}", electronics.len());
    
    let active_items = processor.filter_active();
    println!("Active items: {}", active_items.len());
    
    let total_value = processor.calculate_total_value();
    println!("Total value: {:.2}", total_value);
    
    let avg_value = processor.calculate_average_value();
    println!("Average value: {:.2}", avg_value);
    
    if let Some(max_record) = processor.find_max_value() {
        println!("Most valuable item: {} (${})", max_record.name, max_record.value);
    }
    
    let groups = processor.group_by_category();
    for (category, items) in groups {
        println!("Category '{}' has {} items", category, items.len());
    }
    
    Ok(())
}

fn main() {
    if let Err(e) = process_data_sample() {
        eprintln!("Error processing data: {}", e);
        std::process::exit(1);
    }
}