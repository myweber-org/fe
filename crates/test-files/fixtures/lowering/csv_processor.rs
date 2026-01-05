use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Clone)]
struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

impl Record {
    fn new(id: u32, name: &str, category: &str, value: f64, active: bool) -> Self {
        Record {
            id,
            name: name.to_string(),
            category: category.to_string(),
            value,
            active,
        }
    }

    fn transform_value(&mut self, multiplier: f64) {
        self.value *= multiplier;
    }

    fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0
    }
}

fn load_records(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut csv_reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(reader);

    let mut records = Vec::new();

    for result in csv_reader.deserialize() {
        let record: Record = result?;
        if record.is_valid() {
            records.push(record);
        }
    }

    Ok(records)
}

fn filter_records(records: &[Record], category_filter: &str, min_value: f64) -> Vec<Record> {
    records
        .iter()
        .filter(|r| r.category == category_filter && r.value >= min_value && r.active)
        .cloned()
        .collect()
}

fn process_records(records: &mut [Record], multiplier: f64) {
    for record in records.iter_mut() {
        record.transform_value(multiplier);
    }
}

fn save_records(records: &[Record], output_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::create(output_path)?;
    let writer = BufWriter::new(file);
    let mut csv_writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(writer);

    for record in records {
        csv_writer.serialize(record)?;
    }

    csv_writer.flush()?;
    Ok(())
}

fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    let count = records.len() as f64;
    if count == 0.0 {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let mean = sum / count;
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    let std_dev = variance.sqrt();

    (sum, mean, std_dev)
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/output.csv";
    let target_category = "electronics";
    let value_threshold = 50.0;
    let value_multiplier = 1.15;

    println!("Loading records from {}", input_file);
    let mut records = load_records(input_file)?;
    println!("Loaded {} valid records", records.len());

    println!("Filtering records for category '{}' with value >= {}", target_category, value_threshold);
    let mut filtered = filter_records(&records, target_category, value_threshold);
    println!("Found {} matching records", filtered.len());

    println!("Processing records with multiplier {}", value_multiplier);
    process_records(&mut filtered, value_multiplier);

    let (total, average, deviation) = calculate_statistics(&filtered);
    println!("Statistics - Total: {:.2}, Average: {:.2}, Std Dev: {:.2}", total, average, deviation);

    println!("Saving processed records to {}", output_file);
    save_records(&filtered, output_file)?;
    println!("Processing completed successfully");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = Record::new(1, "Test", "category", 10.0, true);
        assert!(valid_record.is_valid());

        let invalid_name = Record::new(2, "", "category", 10.0, true);
        assert!(!invalid_name.is_valid());

        let invalid_value = Record::new(3, "Test", "category", -5.0, true);
        assert!(!invalid_value.is_valid());
    }

    #[test]
    fn test_value_transformation() {
        let mut record = Record::new(1, "Item", "test", 100.0, true);
        record.transform_value(1.5);
        assert_eq!(record.value, 150.0);
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            Record::new(1, "A", "test", 10.0, true),
            Record::new(2, "B", "test", 20.0, true),
            Record::new(3, "C", "test", 30.0, true),
        ];
        
        let (total, mean, std_dev) = calculate_statistics(&records);
        assert_eq!(total, 60.0);
        assert_eq!(mean, 20.0);
        assert!(std_dev > 0.0);
    }
}