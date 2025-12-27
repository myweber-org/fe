use std::error::Error;
use std::fs::File;
use csv::{Reader, Writer};

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub category: String,
    pub value: f64,
    pub active: bool,
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self, category_filter: Option<&str>) -> f64 {
        let filtered_records: Vec<&DataRecord> = match category_filter {
            Some(category) => self
                .records
                .iter()
                .filter(|record| record.category == category)
                .collect(),
            None => self.records.iter().collect(),
        };

        if filtered_records.is_empty() {
            return 0.0;
        }

        let sum: f64 = filtered_records.iter().map(|record| record.value).sum();
        sum / filtered_records.len() as f64
    }

    pub fn export_filtered_data(
        &self,
        output_path: &str,
        category: &str,
    ) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        let mut wtr = Writer::from_path(output_path)?;

        for record in filtered {
            wtr.serialize(record)?;
        }

        wtr.flush()?;
        Ok(())
    }

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        let values: Vec<f64> = self.records.iter().map(|record| record.value).collect();
        let count = values.len();

        if count == 0 {
            return (0.0, 0.0, 0.0);
        }

        let min = values
            .iter()
            .fold(f64::INFINITY, |acc, &x| if x < acc { x } else { acc });
        let max = values
            .iter()
            .fold(f64::NEG_INFINITY, |acc, &x| if x > acc { x } else { acc });
        let avg = self.calculate_average(None);

        (min, max, avg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        let mut test_records = vec![
            DataRecord {
                id: 1,
                category: "A".to_string(),
                value: 10.5,
                active: true,
            },
            DataRecord {
                id: 2,
                category: "B".to_string(),
                value: 20.3,
                active: false,
            },
            DataRecord {
                id: 3,
                category: "A".to_string(),
                value: 15.7,
                active: true,
            },
        ];

        processor.records = test_records.clone();

        let filtered = processor.filter_by_category("A");
        assert_eq!(filtered.len(), 2);

        let avg_a = processor.calculate_average(Some("A"));
        assert!((avg_a - 13.1).abs() < 0.001);

        let stats = processor.get_statistics();
        assert!((stats.0 - 10.5).abs() < 0.001);
        assert!((stats.1 - 20.3).abs() < 0.001);
    }

    #[test]
    fn test_export_data() {
        let mut processor = DataProcessor::new();
        processor.records = vec![
            DataRecord {
                id: 1,
                category: "Test".to_string(),
                value: 42.0,
                active: true,
            },
        ];

        let temp_file = NamedTempFile::new().unwrap();
        let output_path = temp_file.path().to_str().unwrap();

        let result = processor.export_filtered_data(output_path, "Test");
        assert!(result.is_ok());
    }
}