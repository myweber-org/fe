use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
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

    fn load_from_csv(&mut self, filepath: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        lines.next();

        for line in lines {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() == 4 {
                let id = parts[0].parse::<u32>()?;
                let name = parts[1].to_string();
                let value = parts[2].parse::<f64>()?;
                let category = parts[3].to_string();

                self.records.push(Record {
                    id,
                    name,
                    value,
                    category,
                });
            }
        }

        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    fn aggregate_by_category(&self) -> Vec<(String, f64)> {
        use std::collections::HashMap;

        let mut map: HashMap<String, f64> = HashMap::new();
        for record in &self.records {
            *map.entry(record.category.clone()).or_insert(0.0) += record.value;
        }

        map.into_iter().collect()
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    processor.load_from_csv("data.csv")?;

    println!("Total records: {}", processor.records.len());
    println!("Average value: {:.2}", processor.calculate_average());

    if let Some(max_record) = processor.find_max_value() {
        println!("Max value record: {:?}", max_record);
    }

    let filtered = processor.filter_by_category("premium");
    println!("Premium records: {}", filtered.len());

    let aggregates = processor.aggregate_by_category();
    for (category, total) in aggregates {
        println!("Category {}: total {}", category, total);
    }

    Ok(())
}