
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    category: String,
    value: f64,
    timestamp: String,
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

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    pub fn find_max_value(&self) -> Option<&DataRecord> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn aggregate_by_category(&self) -> Vec<(String, f64)> {
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
            .map(|(category, (sum, count))| (category, sum / count as f64))
            .collect()
    }

    pub fn get_summary(&self) -> DataSummary {
        DataSummary {
            total_records: self.records.len(),
            average_value: self.calculate_average(),
            categories: self.get_unique_categories(),
        }
    }

    fn get_unique_categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self
            .records
            .iter()
            .map(|r| r.category.clone())
            .collect();
        categories.sort();
        categories.dedup();
        categories
    }
}

pub struct DataSummary {
    total_records: usize,
    average_value: f64,
    categories: Vec<String>,
}

impl DataSummary {
    pub fn display(&self) {
        println!("Total Records: {}", self.total_records);
        println!("Average Value: {:.2}", self.average_value);
        println!("Categories: {}", self.categories.join(", "));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,category,value,timestamp").unwrap();
        writeln!(file, "1,electronics,250.50,2023-01-15").unwrap();
        writeln!(file, "2,clothing,89.99,2023-01-16").unwrap();
        writeln!(file, "3,electronics,450.75,2023-01-17").unwrap();
        writeln!(file, "4,clothing,120.25,2023-01-18").unwrap();
        file
    }

    #[test]
    fn test_load_and_filter() {
        let csv_file = create_test_csv();
        let mut processor = DataProcessor::new();
        
        processor.load_from_csv(csv_file.path()).unwrap();
        
        let electronics = processor.filter_by_category("electronics");
        assert_eq!(electronics.len(), 2);
        
        let clothing = processor.filter_by_category("clothing");
        assert_eq!(clothing.len(), 2);
    }

    #[test]
    fn test_calculate_average() {
        let csv_file = create_test_csv();
        let mut processor = DataProcessor::new();
        
        processor.load_from_csv(csv_file.path()).unwrap();
        
        let avg = processor.calculate_average();
        assert!((avg - 227.8725).abs() < 0.001);
    }

    #[test]
    fn test_aggregate_by_category() {
        let csv_file = create_test_csv();
        let mut processor = DataProcessor::new();
        
        processor.load_from_csv(csv_file.path()).unwrap();
        
        let aggregates = processor.aggregate_by_category();
        assert_eq!(aggregates.len(), 2);
    }
}use std::error::Error;
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

    pub fn load_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 5 {
                let record = CsvRecord {
                    id: parts[0].parse()?,
                    name: parts[1].to_string(),
                    category: parts[2].to_string(),
                    value: parts[3].parse()?,
                    active: parts[4].parse().unwrap_or(false),
                };
                self.records.push(record);
            }
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

    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|record| record.value).sum()
    }

    pub fn calculate_average_value(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        self.calculate_total_value() / self.records.len() as f64
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<CsvRecord>> {
        let mut groups = std::collections::HashMap::new();
        
        for record in &self.records {
            groups
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record.clone());
        }
        
        groups
    }

    pub fn find_max_value(&self) -> Option<&CsvRecord> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    pub fn find_min_value(&self) -> Option<&CsvRecord> {
        self.records.iter().min_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn get_all_records(&self) -> &Vec<CsvRecord> {
        &self.records
    }
}

pub fn process_csv_data(file_path: &str) -> Result<(), Box<dyn Error>> {
    let mut processor = CsvProcessor::new();
    processor.load_from_file(file_path)?;
    
    println!("Total records: {}", processor.count_records());
    println!("Total value: {:.2}", processor.calculate_total_value());
    println!("Average value: {:.2}", processor.calculate_average_value());
    
    if let Some(max_record) = processor.find_max_value() {
        println!("Max value record: {:?}", max_record);
    }
    
    if let Some(min_record) = processor.find_min_value() {
        println!("Min value record: {:?}", min_record);
    }
    
    let active_records = processor.filter_active();
    println!("Active records: {}", active_records.len());
    
    let groups = processor.group_by_category();
    for (category, records) in groups {
        println!("Category '{}' has {} records", category, records.len());
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processor() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,category,value,active").unwrap();
        writeln!(temp_file, "1,ItemA,Electronics,100.50,true").unwrap();
        writeln!(temp_file, "2,ItemB,Furniture,75.25,false").unwrap();
        writeln!(temp_file, "3,ItemC,Electronics,150.00,true").unwrap();
        
        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.count_records(), 3);
        
        let electronics = processor.filter_by_category("Electronics");
        assert_eq!(electronics.len(), 2);
        
        let active = processor.filter_active();
        assert_eq!(active.len(), 2);
        
        let total = processor.calculate_total_value();
        assert!((total - 325.75).abs() < 0.01);
        
        let average = processor.calculate_average_value();
        assert!((average - 108.583).abs() < 0.01);
    }
}