use std::collections::HashSet;

pub struct DataCleaner<T> {
    data: Vec<Option<T>>,
}

impl<T> DataCleaner<T>
where
    T: Eq + std::hash::Hash + Clone,
{
    pub fn new(data: Vec<Option<T>>) -> Self {
        DataCleaner { data }
    }

    pub fn remove_nulls(&self) -> Vec<T> {
        self.data
            .iter()
            .filter_map(|item| item.clone())
            .collect()
    }

    pub fn deduplicate(&self) -> Vec<T> {
        let mut seen = HashSet::new();
        self.remove_nulls()
            .into_iter()
            .filter(|item| seen.insert(item.clone()))
            .collect()
    }

    pub fn clean(&self) -> Vec<T> {
        self.deduplicate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_nulls() {
        let data = vec![Some(1), None, Some(2), Some(1), None];
        let cleaner = DataCleaner::new(data);
        let result = cleaner.remove_nulls();
        assert_eq!(result, vec![1, 2, 1]);
    }

    #[test]
    fn test_deduplicate() {
        let data = vec![Some("a"), None, Some("b"), Some("a"), Some("c")];
        let cleaner = DataCleaner::new(data);
        let result = cleaner.deduplicate();
        assert_eq!(result, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_clean() {
        let data = vec![Some(10), None, Some(20), Some(10), None, Some(30)];
        let cleaner = DataCleaner::new(data);
        let result = cleaner.clean();
        assert_eq!(result, vec![10, 20, 30]);
    }
}
use csv::{ReaderBuilder, WriterBuilder};
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};

pub fn remove_duplicates(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);

    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    let mut csv_writer = WriterBuilder::new().has_headers(true).from_writer(writer);

    let headers = csv_reader.headers()?.clone();
    csv_writer.write_record(&headers)?;

    let mut seen_records = HashSet::new();
    for result in csv_reader.records() {
        let record = result?;
        let record_string = record.iter().collect::<Vec<&str>>().join(",");
        
        if !seen_records.contains(&record_string) {
            csv_writer.write_record(&record)?;
            seen_records.insert(record_string);
        }
    }

    csv_writer.flush()?;
    Ok(())
}use std::collections::HashMap;

pub struct DataCleaner {
    filters: Vec<Box<dyn Fn(&HashMap<String, String>) -> bool>>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            filters: Vec::new(),
        }
    }

    pub fn add_filter<F>(&mut self, filter: F)
    where
        F: Fn(&HashMap<String, String>) -> bool + 'static,
    {
        self.filters.push(Box::new(filter));
    }

    pub fn clean(&self, data: Vec<HashMap<String, String>>) -> Vec<HashMap<String, String>> {
        data.into_iter()
            .filter(|entry| self.filters.iter().all(|f| f(entry)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleaner_filters_invalid_data() {
        let mut cleaner = DataCleaner::new();
        
        cleaner.add_filter(|entry| entry.contains_key("id") && !entry.get("id").unwrap().is_empty());
        cleaner.add_filter(|entry| entry.get("status").map_or(false, |s| s == "active"));

        let test_data = vec![
            HashMap::from([("id".to_string(), "".to_string()), ("status".to_string(), "active".to_string())]),
            HashMap::from([("id".to_string(), "123".to_string()), ("status".to_string(), "inactive".to_string())]),
            HashMap::from([("id".to_string(), "456".to_string()), ("status".to_string(), "active".to_string())]),
        ];

        let result = cleaner.clean(test_data);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].get("id").unwrap(), "456");
    }
}