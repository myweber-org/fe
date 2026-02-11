
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub category: String,
    pub value: f64,
    pub active: bool,
}

pub struct CsvProcessor {
    records: Vec<Record>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
            .collect()
    }

    pub fn filter_active(&self) -> Vec<Record> {
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
            0.0
        } else {
            self.calculate_total_value() / self.records.len() as f64
        }
    }

    pub fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn find_min_value(&self) -> Option<&Record> {
        self.records.iter().min_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn get_category_summary(&self) -> Vec<(String, f64, usize)> {
        use std::collections::HashMap;

        let mut category_map: HashMap<String, (f64, usize)> = HashMap::new();

        for record in &self.records {
            let entry = category_map
                .entry(record.category.clone())
                .or_insert((0.0, 0));
            entry.0 += record.value;
            entry.1 += 1;
        }

        category_map
            .into_iter()
            .map(|(category, (total, count))| (category, total, count))
            .collect()
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn clear(&mut self) {
        self.records.clear();
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
            "id,name,category,value,active\n1,ItemA,Electronics,100.5,true\n2,ItemB,Books,25.0,false\n3,ItemC,Electronics,75.25,true"
        )
        .unwrap();
        file
    }

    #[test]
    fn test_load_and_filter() {
        let test_file = create_test_csv();
        let mut processor = CsvProcessor::new();
        processor
            .load_from_file(test_file.path())
            .expect("Failed to load CSV");

        assert_eq!(processor.count_records(), 3);

        let electronics = processor.filter_by_category("Electronics");
        assert_eq!(electronics.len(), 2);

        let active_items = processor.filter_active();
        assert_eq!(active_items.len(), 2);

        let total = processor.calculate_total_value();
        assert!((total - 200.75).abs() < 0.001);

        let avg = processor.calculate_average_value();
        assert!((avg - 66.916666).abs() < 0.001);

        let max_record = processor.find_max_value().unwrap();
        assert_eq!(max_record.id, 1);
        assert!((max_record.value - 100.5).abs() < 0.001);

        let summary = processor.get_category_summary();
        assert_eq!(summary.len(), 2);
    }

    #[test]
    fn test_empty_processor() {
        let processor = CsvProcessor::new();
        assert!(processor.is_empty());
        assert_eq!(processor.calculate_total_value(), 0.0);
        assert_eq!(processor.calculate_average_value(), 0.0);
        assert!(processor.find_max_value().is_none());
        assert!(processor.find_min_value().is_none());
    }
}
use std::error::Error;
use std::fs::File;
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    category: String,
    value: f64,
    timestamp: String,
}

impl DataRecord {
    pub fn new(id: u32, category: String, value: f64, timestamp: String) -> Self {
        DataRecord {
            id,
            category,
            value,
            timestamp,
        }
    }
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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(file);

        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
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

    pub fn aggregate_by_category(&self) -> Vec<(String, f64)> {
        use std::collections::HashMap;

        let mut category_totals: HashMap<String, f64> = HashMap::new();
        let mut category_counts: HashMap<String, usize> = HashMap::new();

        for record in &self.records {
            let total = category_totals.entry(record.category.clone()).or_insert(0.0);
            *total += record.value;

            let count = category_counts.entry(record.category.clone()).or_insert(0);
            *count += 1;
        }

        let mut results = Vec::new();
        for (category, total) in category_totals {
            if let Some(&count) = category_counts.get(&category) {
                results.push((category, total / count as f64));
            }
        }

        results.sort_by(|a, b| a.0.cmp(&b.0));
        results
    }

    pub fn save_filtered_results(&self, category: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        
        let file = File::create(output_path)?;
        let mut wtr = WriterBuilder::new()
            .has_headers(true)
            .from_writer(file);

        for record in filtered {
            wtr.serialize(record)?;
        }

        wtr.flush()?;
        Ok(())
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn get_unique_categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self.records
            .iter()
            .map(|record| record.category.clone())
            .collect();

        categories.sort();
        categories.dedup();
        categories
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,category,value,timestamp").unwrap();
        writeln!(temp_file, "1,electronics,299.99,2023-01-15").unwrap();
        writeln!(temp_file, "2,clothing,49.95,2023-01-16").unwrap();
        writeln!(temp_file, "3,electronics,599.99,2023-01-17").unwrap();
        writeln!(temp_file, "4,clothing,79.99,2023-01-18").unwrap();
        temp_file
    }

    #[test]
    fn test_load_and_filter() {
        let temp_file = create_test_csv();
        let mut processor = DataProcessor::new();
        
        processor.load_from_csv(temp_file.path().to_str().unwrap()).unwrap();
        
        let electronics = processor.filter_by_category("electronics");
        assert_eq!(electronics.len(), 2);
        
        let clothing = processor.filter_by_category("clothing");
        assert_eq!(clothing.len(), 2);
    }

    #[test]
    fn test_calculate_average() {
        let temp_file = create_test_csv();
        let mut processor = DataProcessor::new();
        
        processor.load_from_csv(temp_file.path().to_str().unwrap()).unwrap();
        
        let avg = processor.calculate_average();
        let expected = (299.99 + 49.95 + 599.99 + 79.99) / 4.0;
        assert!((avg - expected).abs() < 0.001);
    }

    #[test]
    fn test_aggregate_by_category() {
        let temp_file = create_test_csv();
        let mut processor = DataProcessor::new();
        
        processor.load_from_csv(temp_file.path().to_str().unwrap()).unwrap();
        
        let aggregates = processor.aggregate_by_category();
        assert_eq!(aggregates.len(), 2);
        
        let electronics_avg = aggregates.iter()
            .find(|(cat, _)| cat == "electronics")
            .map(|(_, avg)| *avg)
            .unwrap();
        
        let expected_electronics_avg = (299.99 + 599.99) / 2.0;
        assert!((electronics_avg - expected_electronics_avg).abs() < 0.001);
    }
}