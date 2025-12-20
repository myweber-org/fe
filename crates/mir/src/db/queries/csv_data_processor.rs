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

    fn load_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
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

    fn summary_statistics(&self) -> String {
        let total = self.calculate_total_value();
        let average = self.calculate_average_value();
        let active_count = self.filter_active().len();
        let max_record = self.find_max_value();

        let max_info = match max_record {
            Some(record) => format!("{} (ID: {})", record.value, record.id),
            None => "None".to_string(),
        };

        format!(
            "Total Records: {}\nActive Records: {}\nTotal Value: {:.2}\nAverage Value: {:.2}\nMax Value: {}",
            self.records.len(),
            active_count,
            total,
            average,
            max_info
        )
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    match processor.load_from_file("data.csv") {
        Ok(_) => {
            println!("Data loaded successfully!");
            println!("\nSummary Statistics:");
            println!("{}", processor.summary_statistics());
            
            println!("\nRecords in 'Electronics' category:");
            let electronics = processor.filter_by_category("Electronics");
            for record in electronics {
                println!("ID: {}, Name: {}, Value: {:.2}", record.id, record.name, record.value);
            }
            
            println!("\nGrouped by category:");
            let groups = processor.group_by_category();
            for (category, records) in groups {
                println!("Category: {} ({} records)", category, records.len());
            }
        }
        Err(e) => {
            eprintln!("Error loading data: {}", e);
            println!("Creating sample data for demonstration...");
            
            processor.records = vec![
                Record {
                    id: 1,
                    name: "Laptop".to_string(),
                    category: "Electronics".to_string(),
                    value: 999.99,
                    active: true,
                },
                Record {
                    id: 2,
                    name: "Desk".to_string(),
                    category: "Furniture".to_string(),
                    value: 299.50,
                    active: true,
                },
                Record {
                    id: 3,
                    name: "Monitor".to_string(),
                    category: "Electronics".to_string(),
                    value: 199.99,
                    active: false,
                },
                Record {
                    id: 4,
                    name: "Chair".to_string(),
                    category: "Furniture".to_string(),
                    value: 149.99,
                    active: true,
                },
            ];
            
            println!("\nSummary Statistics (Sample Data):");
            println!("{}", processor.summary_statistics());
        }
    }
    
    Ok(())
}