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

    fn aggregate_by_category(&self) -> HashMap<String, (f64, usize)> {
        let mut aggregates = HashMap::new();

        for record in &self.records {
            if !record.active {
                continue;
            }

            let entry = aggregates
                .entry(record.category.clone())
                .or_insert((0.0, 0));

            entry.0 += record.value;
            entry.1 += 1;
        }

        aggregates
    }

    fn calculate_average_value(&self) -> Option<f64> {
        let active_records: Vec<&Record> = self.records.iter().filter(|r| r.active).collect();

        if active_records.is_empty() {
            return None;
        }

        let total: f64 = active_records.iter().map(|r| r.value).sum();
        Some(total / active_records.len() as f64)
    }

    fn find_max_value_record(&self) -> Option<&Record> {
        self.records
            .iter()
            .filter(|r| r.active)
            .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
    }
}

fn process_data_file(input_path: &str) -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    processor.load_from_file(input_path)?;

    println!("Total records loaded: {}", processor.records.len());

    if let Some(avg) = processor.calculate_average_value() {
        println!("Average value of active records: {:.2}", avg);
    }

    let aggregates = processor.aggregate_by_category();
    for (category, (total, count)) in aggregates {
        println!(
            "Category '{}': {} records, total value: {:.2}",
            category, count, total
        );
    }

    if let Some(max_record) = processor.find_max_value_record() {
        println!(
            "Record with maximum value: ID {}, Category {}, Value {:.2}",
            max_record.id, max_record.category, max_record.value
        );
    }

    let filtered = processor.filter_by_category("premium");
    println!("Premium active records: {}", filtered.len());

    Ok(())
}

fn main() {
    let input_file = "data/sample.csv";
    match process_data_file(input_file) {
        Ok(_) => println!("Data processing completed successfully"),
        Err(e) => eprintln!("Error processing data: {}", e),
    }
}