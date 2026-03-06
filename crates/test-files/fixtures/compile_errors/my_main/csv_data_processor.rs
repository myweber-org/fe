
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TransactionRecord {
    pub id: u32,
    pub customer_id: String,
    pub amount: f64,
    pub currency: String,
    pub status: String,
    pub timestamp: String,
}

pub struct TransactionProcessor {
    records: Vec<TransactionRecord>,
}

impl TransactionProcessor {
    pub fn new() -> Self {
        TransactionProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut csv_reader = csv::Reader::from_reader(reader);

        for result in csv_reader.deserialize() {
            let record: TransactionRecord = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_status(&self, status: &str) -> Vec<TransactionRecord> {
        self.records
            .iter()
            .filter(|record| record.status == status)
            .cloned()
            .collect()
    }

    pub fn filter_by_currency(&self, currency: &str) -> Vec<TransactionRecord> {
        self.records
            .iter()
            .filter(|record| record.currency == currency)
            .cloned()
            .collect()
    }

    pub fn calculate_total_amount(&self) -> f64 {
        self.records.iter().map(|record| record.amount).sum()
    }

    pub fn calculate_average_amount(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        self.calculate_total_amount() / self.records.len() as f64
    }

    pub fn find_customer_transactions(&self, customer_id: &str) -> Vec<TransactionRecord> {
        self.records
            .iter()
            .filter(|record| record.customer_id == customer_id)
            .cloned()
            .collect()
    }

    pub fn save_filtered_to_csv<P: AsRef<Path>>(
        &self,
        records: &[TransactionRecord],
        path: P,
    ) -> Result<(), Box<dyn Error>> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        let mut csv_writer = csv::Writer::from_writer(writer);

        for record in records {
            csv_writer.serialize(record)?;
        }

        csv_writer.flush()?;
        Ok(())
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn get_unique_customers(&self) -> Vec<String> {
        let mut customers: Vec<String> = self
            .records
            .iter()
            .map(|record| record.customer_id.clone())
            .collect();
        customers.sort();
        customers.dedup();
        customers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_transaction_processor() {
        let mut processor = TransactionProcessor::new();
        let test_data = vec![
            TransactionRecord {
                id: 1,
                customer_id: "CUST001".to_string(),
                amount: 100.0,
                currency: "USD".to_string(),
                status: "COMPLETED".to_string(),
                timestamp: "2024-01-15T10:30:00Z".to_string(),
            },
            TransactionRecord {
                id: 2,
                customer_id: "CUST002".to_string(),
                amount: 200.0,
                currency: "EUR".to_string(),
                status: "PENDING".to_string(),
                timestamp: "2024-01-15T11:45:00Z".to_string(),
            },
            TransactionRecord {
                id: 3,
                customer_id: "CUST001".to_string(),
                amount: 150.0,
                currency: "USD".to_string(),
                status: "COMPLETED".to_string(),
                timestamp: "2024-01-15T12:15:00Z".to_string(),
            },
        ];

        processor.records = test_data;

        assert_eq!(processor.get_record_count(), 3);
        assert_eq!(processor.calculate_total_amount(), 450.0);
        assert_eq!(processor.calculate_average_amount(), 150.0);

        let completed = processor.filter_by_status("COMPLETED");
        assert_eq!(completed.len(), 2);

        let usd_transactions = processor.filter_by_currency("USD");
        assert_eq!(usd_transactions.len(), 2);

        let cust001_transactions = processor.find_customer_transactions("CUST001");
        assert_eq!(cust001_transactions.len(), 2);

        let unique_customers = processor.get_unique_customers();
        assert_eq!(unique_customers.len(), 2);
        assert!(unique_customers.contains(&"CUST001".to_string()));
        assert!(unique_customers.contains(&"CUST002".to_string()));
    }

    #[test]
    fn test_save_to_csv() {
        let processor = TransactionProcessor::new();
        let test_records = vec![TransactionRecord {
            id: 1,
            customer_id: "TEST001".to_string(),
            amount: 50.0,
            currency: "GBP".to_string(),
            status: "COMPLETED".to_string(),
            timestamp: "2024-01-15T14:20:00Z".to_string(),
        }];

        let temp_file = NamedTempFile::new().unwrap();
        let result = processor.save_filtered_to_csv(&test_records, temp_file.path());
        assert!(result.is_ok());
    }
}use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

fn load_csv(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        records.push(record);
    }

    Ok(records)
}

fn filter_active_records(records: Vec<Record>) -> Vec<Record> {
    records.into_iter().filter(|r| r.active).collect()
}

fn calculate_category_totals(records: Vec<Record>) -> Vec<(String, f64)> {
    let mut totals = std::collections::HashMap::new();

    for record in records {
        *totals.entry(record.category.clone()).or_insert(0.0) += record.value;
    }

    totals.into_iter().collect()
}

fn save_results(results: Vec<(String, f64)>, output_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::create(output_path)?;
    let mut writer = Writer::from_writer(file);

    writer.write_record(&["Category", "Total"])?;

    for (category, total) in results {
        writer.write_record(&[category, total.to_string()])?;
    }

    writer.flush()?;
    Ok(())
}

fn process_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let records = load_csv(input_path)?;
    let active_records = filter_active_records(records);
    let category_totals = calculate_category_totals(active_records);
    save_results(category_totals, output_path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_filter_active_records() {
        let records = vec![
            Record {
                id: 1,
                name: "Item A".to_string(),
                category: "Electronics".to_string(),
                value: 100.0,
                active: true,
            },
            Record {
                id: 2,
                name: "Item B".to_string(),
                category: "Books".to_string(),
                value: 50.0,
                active: false,
            },
        ];

        let active = filter_active_records(records);
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].name, "Item A");
    }

    #[test]
    fn test_calculate_category_totals() {
        let records = vec![
            Record {
                id: 1,
                name: "Item A".to_string(),
                category: "Electronics".to_string(),
                value: 100.0,
                active: true,
            },
            Record {
                id: 2,
                name: "Item B".to_string(),
                category: "Electronics".to_string(),
                value: 200.0,
                active: true,
            },
        ];

        let totals = calculate_category_totals(records);
        assert_eq!(totals.len(), 1);
        assert_eq!(totals[0].0, "Electronics");
        assert_eq!(totals[0].1, 300.0);
    }

    #[test]
    fn test_process_csv_data() -> Result<(), Box<dyn Error>> {
        let input_data = "id,name,category,value,active\n1,Item A,Electronics,100.0,true\n2,Item B,Books,50.0,false";
        
        let input_file = NamedTempFile::new()?;
        std::fs::write(input_file.path(), input_data)?;
        
        let output_file = NamedTempFile::new()?;
        
        process_csv_data(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
        )?;

        let output_content = std::fs::read_to_string(output_file.path())?;
        assert!(output_content.contains("Electronics"));
        assert!(output_content.contains("100"));

        Ok(())
    }
}