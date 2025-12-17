
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
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        let mut records = Vec::new();

        for result in rdr.deserialize() {
            let record: Record = result?;
            records.push(record);
        }

        Ok(CsvProcessor { records })
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

    pub fn find_max_value_record(&self) -> Option<Record> {
        self.records
            .iter()
            .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
            .cloned()
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<Record>> {
        let mut groups = std::collections::HashMap::new();
        
        for record in &self.records {
            groups
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record.clone());
        }
        
        groups
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn get_records(&self) -> &[Record] {
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
        writeln!(file, "id,name,category,value,active").unwrap();
        writeln!(file, "1,ItemA,Electronics,150.5,true").unwrap();
        writeln!(file, "2,ItemB,Books,25.0,true").unwrap();
        writeln!(file, "3,ItemC,Electronics,300.0,false").unwrap();
        writeln!(file, "4,ItemD,Books,45.75,true").unwrap();
        file
    }

    #[test]
    fn test_csv_loading() {
        let file = create_test_csv();
        let processor = CsvProcessor::from_file(file.path()).unwrap();
        assert_eq!(processor.count_records(), 4);
    }

    #[test]
    fn test_filter_by_category() {
        let file = create_test_csv();
        let processor = CsvProcessor::from_file(file.path()).unwrap();
        let electronics = processor.filter_by_category("Electronics");
        assert_eq!(electronics.len(), 2);
    }

    #[test]
    fn test_filter_active() {
        let file = create_test_csv();
        let processor = CsvProcessor::from_file(file.path()).unwrap();
        let active = processor.filter_active();
        assert_eq!(active.len(), 3);
    }

    #[test]
    fn test_calculate_total() {
        let file = create_test_csv();
        let processor = CsvProcessor::from_file(file.path()).unwrap();
        let total = processor.calculate_total_value();
        assert_eq!(total, 521.25);
    }

    #[test]
    fn test_find_max_value() {
        let file = create_test_csv();
        let processor = CsvProcessor::from_file(file.path()).unwrap();
        let max_record = processor.find_max_value_record().unwrap();
        assert_eq!(max_record.id, 3);
        assert_eq!(max_record.value, 300.0);
    }
}