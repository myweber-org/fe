
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Clone)]
struct DataRecord {
    id: u32,
    category: String,
    value: f64,
    timestamp: String,
}

impl DataRecord {
    fn new(id: u32, category: String, value: f64, timestamp: String) -> Self {
        Self {
            id,
            category,
            value,
            timestamp,
        }
    }
}

struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(reader);

        for result in csv_reader.records() {
            let record = result?;
            if record.len() >= 4 {
                let id: u32 = record[0].parse().unwrap_or(0);
                let category = record[1].to_string();
                let value: f64 = record[2].parse().unwrap_or(0.0);
                let timestamp = record[3].to_string();

                self.records.push(DataRecord::new(id, category, value, timestamp));
            }
        }

        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    fn calculate_average(&self, category: Option<&str>) -> f64 {
        let filtered_records: Vec<&DataRecord> = match category {
            Some(cat) => self.filter_by_category(cat),
            None => self.records.iter().collect(),
        };

        if filtered_records.is_empty() {
            return 0.0;
        }

        let sum: f64 = filtered_records.iter().map(|r| r.value).sum();
        sum / filtered_records.len() as f64
    }

    fn save_filtered_to_csv(&self, category: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        
        let file = File::create(output_path)?;
        let writer = BufWriter::new(file);
        let mut csv_writer = WriterBuilder::new()
            .has_headers(true)
            .from_writer(writer);

        csv_writer.write_record(&["ID", "Category", "Value", "Timestamp"])?;

        for record in filtered {
            csv_writer.write_record(&[
                record.id.to_string(),
                record.category.clone(),
                record.value.to_string(),
                record.timestamp.clone(),
            ])?;
        }

        csv_writer.flush()?;
        Ok(())
    }

    fn get_summary(&self) -> String {
        let total_records = self.records.len();
        let categories: Vec<String> = self.records
            .iter()
            .map(|r| r.category.clone())
            .collect();
        
        let unique_categories: std::collections::HashSet<String> = categories.into_iter().collect();
        
        format!(
            "Total records: {}, Unique categories: {}, Overall average: {:.2}",
            total_records,
            unique_categories.len(),
            self.calculate_average(None)
        )
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    match processor.load_from_csv("input_data.csv") {
        Ok(_) => println!("Data loaded successfully"),
        Err(e) => eprintln!("Error loading data: {}", e),
    }

    println!("{}", processor.get_summary());

    let tech_records = processor.filter_by_category("Technology");
    println!("Technology records: {}", tech_records.len());

    let tech_avg = processor.calculate_average(Some("Technology"));
    println!("Technology average value: {:.2}", tech_avg);

    processor.save_filtered_to_csv("Technology", "tech_data.csv")?;
    println!("Filtered data saved to tech_data.csv");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, "Test".to_string(), 100.0, "2024-01-01".to_string());
        assert_eq!(record.id, 1);
        assert_eq!(record.category, "Test");
        assert_eq!(record.value, 100.0);
    }

    #[test]
    fn test_empty_processor() {
        let processor = DataProcessor::new();
        assert_eq!(processor.records.len(), 0);
    }

    #[test]
    fn test_average_calculation() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, "A".to_string(), 10.0, "".to_string()));
        processor.records.push(DataRecord::new(2, "A".to_string(), 20.0, "".to_string()));
        
        let avg = processor.calculate_average(Some("A"));
        assert_eq!(avg, 15.0);
    }
}