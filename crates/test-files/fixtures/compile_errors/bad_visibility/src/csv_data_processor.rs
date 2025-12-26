
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub category: String,
    pub value: f64,
    pub active: bool,
}

pub struct CsvProcessor {
    records: Vec<CsvRecord>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_file(&mut self, filepath: &str) -> Result<usize, Box<dyn Error>> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 5 {
                let record = CsvRecord {
                    id: parts[0].parse().unwrap_or(0),
                    name: parts[1].to_string(),
                    category: parts[2].to_string(),
                    value: parts[3].parse().unwrap_or(0.0),
                    active: parts[4].parse().unwrap_or(false),
                };
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<CsvRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
            .collect()
    }

    pub fn filter_active(&self) -> Vec<CsvRecord> {
        self.records
            .iter()
            .filter(|r| r.active)
            .cloned()
            .collect()
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }

    pub fn calculate_average_value(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        self.calculate_total_value() / self.records.len() as f64
    }

    pub fn group_by_category(&self) -> HashMap<String, Vec<CsvRecord>> {
        let mut groups: HashMap<String, Vec<CsvRecord>> = HashMap::new();

        for record in &self.records {
            groups
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record.clone());
        }

        groups
    }

    pub fn find_max_value_record(&self) -> Option<CsvRecord> {
        self.records
            .iter()
            .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
            .cloned()
    }

    pub fn find_min_value_record(&self) -> Option<CsvRecord> {
        self.records
            .iter()
            .min_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
            .cloned()
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn get_all_records(&self) -> &Vec<CsvRecord> {
        &self.records
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
            "id,name,category,value,active\n1,ItemA,Electronics,250.5,true\n2,ItemB,Furniture,150.0,false\n3,ItemC,Electronics,300.75,true\n4,ItemD,Books,45.25,true"
        )
        .unwrap();
        file
    }

    #[test]
    fn test_load_and_count() {
        let test_file = create_test_csv();
        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(test_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.count_records(), 4);
    }

    #[test]
    fn test_filter_by_category() {
        let test_file = create_test_csv();
        let mut processor = CsvProcessor::new();
        processor
            .load_from_file(test_file.path().to_str().unwrap())
            .unwrap();
        let electronics = processor.filter_by_category("Electronics");
        assert_eq!(electronics.len(), 2);
    }

    #[test]
    fn test_calculate_total() {
        let test_file = create_test_csv();
        let mut processor = CsvProcessor::new();
        processor
            .load_from_file(test_file.path().to_str().unwrap())
            .unwrap();
        let total = processor.calculate_total_value();
        assert!((total - 746.5).abs() < 0.001);
    }

    #[test]
    fn test_find_max_value() {
        let test_file = create_test_csv();
        let mut processor = CsvProcessor::new();
        processor
            .load_from_file(test_file.path().to_str().unwrap())
            .unwrap();
        let max_record = processor.find_max_value_record();
        assert!(max_record.is_some());
        assert_eq!(max_record.unwrap().value, 300.75);
    }
}