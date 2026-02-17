
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
}