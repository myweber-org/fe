use std::collections::HashSet;
use std::error::Error;

pub struct DataCleaner {
    data: Vec<Vec<String>>,
}

impl DataCleaner {
    pub fn new(data: Vec<Vec<String>>) -> Self {
        DataCleaner { data }
    }

    pub fn remove_null_rows(&mut self) -> &mut Self {
        self.data.retain(|row| !row.iter().any(|cell| cell.trim().is_empty()));
        self
    }

    pub fn deduplicate(&mut self) -> &mut Self {
        let mut seen = HashSet::new();
        self.data.retain(|row| {
            let key: String = row.iter().map(|s| s.trim()).collect();
            seen.insert(key)
        });
        self
    }

    pub fn get_cleaned_data(&self) -> &Vec<Vec<String>> {
        &self.data
    }

    pub fn process_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
        let mut rdr = csv::Reader::from_path(input_path)?;
        let mut data = Vec::new();

        for result in rdr.records() {
            let record = result?;
            let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
            data.push(row);
        }

        let mut cleaner = DataCleaner::new(data);
        cleaner.remove_null_rows().deduplicate();

        let mut wtr = csv::Writer::from_path(output_path)?;
        for row in cleaner.get_cleaned_data() {
            wtr.write_record(row)?;
        }
        wtr.flush()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_null_rows() {
        let data = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["".to_string(), "c".to_string()],
            vec!["d".to_string(), "".to_string()],
        ];
        let mut cleaner = DataCleaner::new(data);
        cleaner.remove_null_rows();
        assert_eq!(cleaner.get_cleaned_data().len(), 1);
    }

    #[test]
    fn test_deduplicate() {
        let data = vec![
            vec!["x".to_string(), "y".to_string()],
            vec!["x".to_string(), "y".to_string()],
            vec!["a".to_string(), "b".to_string()],
        ];
        let mut cleaner = DataCleaner::new(data);
        cleaner.deduplicate();
        assert_eq!(cleaner.get_cleaned_data().len(), 2);
    }
}