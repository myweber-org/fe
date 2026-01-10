
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

#[derive(Debug, Clone)]
struct DataRecord {
    id: u32,
    name: String,
    category: String,
    value: f64,
    timestamp: String,
}

impl DataRecord {
    fn from_csv_line(line: &str) -> Result<Self, Box<dyn Error>> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 5 {
            return Err("Invalid CSV line format".into());
        }

        Ok(DataRecord {
            id: parts[0].parse()?,
            name: parts[1].to_string(),
            category: parts[2].to_string(),
            value: parts[3].parse()?,
            timestamp: parts[4].to_string(),
        })
    }

    fn to_csv_line(&self) -> String {
        format!("{},{},{},{},{}", self.id, self.name, self.category, self.value, self.timestamp)
    }
}

struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
                continue;
            }

            let record = DataRecord::from_csv_line(&line)?;
            self.records.push(record);
        }

        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    fn aggregate_by_category(&self) -> Vec<(String, f64, usize)> {
        use std::collections::HashMap;

        let mut aggregates: HashMap<String, (f64, usize)> = HashMap::new();

        for record in &self.records {
            let entry = aggregates
                .entry(record.category.clone())
                .or_insert((0.0, 0));
            entry.0 += record.value;
            entry.1 += 1;
        }

        aggregates
            .into_iter()
            .map(|(category, (total, count))| (category, total, count))
            .collect()
    }

    fn save_filtered_results<P: AsRef<Path>>(
        &self,
        category: &str,
        output_path: P,
    ) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        let mut file = File::create(output_path)?;

        writeln!(file, "id,name,category,value,timestamp")?;

        for record in filtered {
            writeln!(file, "{}", record.to_csv_line())?;
        }

        Ok(())
    }

    fn calculate_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;

        let variance: f64 = values
            .iter()
            .map(|value| {
                let diff = mean - *value;
                diff * diff
            })
            .sum::<f64>()
            / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }
}

fn process_data_file() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    processor.load_from_file("input_data.csv")?;

    let electronics_records = processor.filter_by_category("electronics");
    println!("Found {} electronics records", electronics_records.len());

    let aggregates = processor.aggregate_by_category();
    for (category, total, count) in aggregates {
        println!("Category: {}, Total: {:.2}, Count: {}", category, total, count);
    }

    let (mean, variance, std_dev) = processor.calculate_statistics();
    println!("Statistics - Mean: {:.2}, Variance: {:.2}, Std Dev: {:.2}", mean, variance, std_dev);

    processor.save_filtered_results("electronics", "electronics_data.csv")?;

    Ok(())
}

fn main() {
    if let Err(e) = process_data_file() {
        eprintln!("Error processing data: {}", e);
        std::process::exit(1);
    }
}