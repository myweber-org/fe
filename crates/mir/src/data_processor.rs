
use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        sum / self.records.len() as f64
    }

    pub fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn get_statistics(&self) -> (usize, f64, f64, f64) {
        let count = self.records.len();
        let avg = self.calculate_average();

        let min = self
            .records
            .iter()
            .map(|r| r.value)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        let max = self
            .records
            .iter()
            .map(|r| r.value)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        (count, avg, min, max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            "id,name,value,category\n1,ItemA,10.5,Alpha\n2,ItemB,15.3,Beta\n3,ItemC,8.7,Alpha"
        )
        .unwrap();
        file
    }

    #[test]
    fn test_load_and_filter() {
        let test_file = create_test_csv();
        let mut processor = DataProcessor::new();

        processor
            .load_from_csv(test_file.path().to_str().unwrap())
            .unwrap();

        let alpha_records = processor.filter_by_category("Alpha");
        assert_eq!(alpha_records.len(), 2);

        let beta_records = processor.filter_by_category("Beta");
        assert_eq!(beta_records.len(), 1);
    }

    #[test]
    fn test_calculations() {
        let test_file = create_test_csv();
        let mut processor = DataProcessor::new();

        processor
            .load_from_csv(test_file.path().to_str().unwrap())
            .unwrap();

        let avg = processor.calculate_average();
        assert!((avg - 11.5).abs() < 0.01);

        let max_record = processor.find_max_value().unwrap();
        assert_eq!(max_record.name, "ItemB");
        assert!((max_record.value - 15.3).abs() < 0.01);
    }
}