use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    fn new() -> Self {
        DataProcessor { records: Vec::new() }
    }

    fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(file);

        for result in rdr.deserialize() {
            let record: Record = result?;
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

    fn aggregate_values(&self) -> f64 {
        self.records
            .iter()
            .filter(|record| record.active)
            .map(|record| record.value)
            .sum()
    }

    fn save_filtered_results<P: AsRef<Path>>(
        &self,
        category: &str,
        output_path: P,
    ) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        let mut wtr = WriterBuilder::new()
            .has_headers(true)
            .from_path(output_path)?;

        for record in filtered {
            wtr.serialize(record)?;
        }

        wtr.flush()?;
        Ok(())
    }

    fn calculate_statistics(&self) -> (f64, f64, f64) {
        let active_values: Vec<f64> = self
            .records
            .iter()
            .filter(|r| r.active)
            .map(|r| r.value)
            .collect();

        if active_values.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = active_values.iter().sum();
        let count = active_values.len() as f64;
        let average = sum / count;
        let max = active_values
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let min = active_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));

        (average, max, min)
    }
}

fn process_data_file(input_path: &str, output_path: &str, category: &str) -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    processor.load_from_file(input_path)?;

    let stats = processor.calculate_statistics();
    println!("Statistics - Average: {:.2}, Max: {:.2}, Min: {:.2}", stats.0, stats.1, stats.2);

    let total = processor.aggregate_values();
    println!("Total active values: {:.2}", total);

    processor.save_filtered_results(category, output_path)?;
    println!("Filtered data saved to: {}", output_path);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        processor.records.push(Record {
            id: 1,
            name: "Item1".to_string(),
            category: "A".to_string(),
            value: 100.0,
            active: true,
        });

        processor.records.push(Record {
            id: 2,
            name: "Item2".to_string(),
            category: "B".to_string(),
            value: 200.0,
            active: false,
        });

        let filtered = processor.filter_by_category("A");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "Item1");

        let total = processor.aggregate_values();
        assert_eq!(total, 100.0);
    }
}