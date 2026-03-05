use std::error::Error;
use std::fs::File;
use csv::{Reader, Writer};

#[derive(Debug, Clone)]
struct DataRecord {
    id: u32,
    category: String,
    value: f64,
    active: bool,
}

impl DataRecord {
    fn new(id: u32, category: String, value: f64, active: bool) -> Self {
        Self {
            id,
            category,
            value,
            active,
        }
    }
}

fn load_csv_data(file_path: &str) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: DataRecord = result?;
        records.push(record);
    }

    Ok(records)
}

fn filter_active_records(records: &[DataRecord]) -> Vec<DataRecord> {
    records
        .iter()
        .filter(|r| r.active)
        .cloned()
        .collect()
}

fn calculate_category_averages(records: &[DataRecord]) -> Vec<(String, f64)> {
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

fn write_results_to_csv(
    file_path: &str,
    averages: &[(String, f64)],
) -> Result<(), Box<dyn Error>> {
    let file = File::create(file_path)?;
    let mut writer = Writer::from_writer(file);

    writer.write_record(&["Category", "AverageValue"])?;

    for (category, average) in averages {
        writer.write_record(&[category, &average.to_string()])?;
    }

    writer.flush()?;
    Ok(())
}

fn process_data_pipeline(input_file: &str, output_file: &str) -> Result<(), Box<dyn Error>> {
    let all_records = load_csv_data(input_file)?;
    let active_records = filter_active_records(&all_records);
    let category_averages = calculate_category_averages(&active_records);
    write_results_to_csv(output_file, &category_averages)?;

    println!("Processed {} records", all_records.len());
    println!("Found {} active records", active_records.len());
    println!("Generated averages for {} categories", category_averages.len());

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_path = "data/input.csv";
    let output_path = "data/output.csv";

    match process_data_pipeline(input_path, output_path) {
        Ok(()) => println!("Data processing completed successfully"),
        Err(e) => eprintln!("Error processing data: {}", e),
    }

    Ok(())
}
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};

#[derive(Debug, Clone)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub category: String,
    pub value: f64,
    pub active: bool,
}

impl CsvRecord {
    pub fn new(id: u32, name: String, category: String, value: f64, active: bool) -> Self {
        Self {
            id,
            name,
            category,
            value,
            active,
        }
    }
}

pub struct CsvProcessor {
    records: Vec<CsvRecord>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    pub fn load_from_file(&mut self, filepath: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);
        let mut rdr = csv::Reader::from_reader(reader);

        for result in rdr.deserialize() {
            let record: CsvRecord = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .cloned()
            .collect()
    }

    pub fn filter_active(&self) -> Vec<CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.active)
            .cloned()
            .collect()
    }

    pub fn aggregate_by_category(&self) -> Vec<(String, f64, usize)> {
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

    pub fn calculate_average_value(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn save_filtered_to_file(
        &self,
        filepath: &str,
        records: &[CsvRecord],
    ) -> Result<(), Box<dyn Error>> {
        let file = File::create(filepath)?;
        let writer = BufWriter::new(file);
        let mut wtr = csv::Writer::from_writer(writer);

        for record in records {
            wtr.serialize(record)?;
        }

        wtr.flush()?;
        Ok(())
    }

    pub fn get_statistics(&self) -> (usize, Option<f64>, Option<f64>) {
        let count = self.records.len();

        let min_value = self
            .records
            .iter()
            .map(|record| record.value)
            .min_by(|a, b| a.partial_cmp(b).unwrap());

        let max_value = self
            .records
            .iter()
            .map(|record| record.value)
            .max_by(|a, b| a.partial_cmp(b).unwrap());

        (count, min_value, max_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processor_operations() {
        let mut processor = CsvProcessor::new();

        let records = vec![
            CsvRecord::new(1, "Item1".to_string(), "Electronics".to_string(), 299.99, true),
            CsvRecord::new(2, "Item2".to_string(), "Books".to_string(), 19.99, true),
            CsvRecord::new(3, "Item3".to_string(), "Electronics".to_string(), 599.99, false),
            CsvRecord::new(4, "Item4".to_string(), "Clothing".to_string(), 49.99, true),
        ];

        processor.records = records;

        let electronics = processor.filter_by_category("Electronics");
        assert_eq!(electronics.len(), 2);

        let active_items = processor.filter_active();
        assert_eq!(active_items.len(), 3);

        let aggregates = processor.aggregate_by_category();
        assert_eq!(aggregates.len(), 3);

        let avg = processor.calculate_average_value().unwrap();
        assert!(avg > 0.0);

        let (count, min, max) = processor.get_statistics();
        assert_eq!(count, 4);
        assert!(min.is_some());
        assert!(max.is_some());
    }

    #[test]
    fn test_file_operations() -> Result<(), Box<dyn Error>> {
        let mut processor = CsvProcessor::new();

        let temp_file = NamedTempFile::new()?;
        let test_data = "id,name,category,value,active\n1,Test1,CategoryA,100.0,true\n2,Test2,CategoryB,200.0,false";

        std::fs::write(temp_file.path(), test_data)?;
        processor.load_from_file(temp_file.path().to_str().unwrap())?;

        assert_eq!(processor.records.len(), 2);

        let filtered = processor.filter_active();
        assert_eq!(filtered.len(), 1);

        let output_file = NamedTempFile::new()?;
        processor.save_filtered_to_file(
            output_file.path().to_str().unwrap(),
            &filtered,
        )?;

        let saved_content = std::fs::read_to_string(output_file.path())?;
        assert!(saved_content.contains("Test1"));

        Ok(())
    }
}