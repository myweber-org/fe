use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use serde::{Deserialize, Serialize};
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Deserialize, Serialize)]
struct Transaction {
    id: u32,
    customer_id: u32,
    amount: f64,
    currency: String,
    status: String,
    timestamp: String,
}

struct TransactionProcessor {
    transactions: Vec<Transaction>,
}

impl TransactionProcessor {
    fn new() -> Self {
        TransactionProcessor {
            transactions: Vec::new(),
        }
    }

    fn load_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(reader);

        for result in csv_reader.deserialize() {
            let transaction: Transaction = result?;
            self.transactions.push(transaction);
        }

        Ok(())
    }

    fn filter_by_status(&self, status: &str) -> Vec<&Transaction> {
        self.transactions
            .iter()
            .filter(|t| t.status == status)
            .collect()
    }

    fn calculate_total_amount(&self, currency: &str) -> f64 {
        self.transactions
            .iter()
            .filter(|t| t.currency == currency)
            .map(|t| t.amount)
            .sum()
    }

    fn save_filtered_transactions(
        &self,
        file_path: &str,
        status_filter: &str,
    ) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_status(status_filter);
        let file = File::create(file_path)?;
        let writer = BufWriter::new(file);
        let mut csv_writer = WriterBuilder::new().from_writer(writer);

        for transaction in filtered {
            csv_writer.serialize(transaction)?;
        }

        csv_writer.flush()?;
        Ok(())
    }

    fn get_customer_summary(&self, customer_id: u32) -> (usize, f64) {
        let customer_transactions: Vec<&Transaction> = self
            .transactions
            .iter()
            .filter(|t| t.customer_id == customer_id)
            .collect();

        let total_amount: f64 = customer_transactions.iter().map(|t| t.amount).sum();
        (customer_transactions.len(), total_amount)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut processor = TransactionProcessor::new();
    
    processor.load_from_file("transactions.csv")?;
    
    let completed_transactions = processor.filter_by_status("completed");
    println!("Found {} completed transactions", completed_transactions.len());
    
    let usd_total = processor.calculate_total_amount("USD");
    println!("Total USD amount: {:.2}", usd_total);
    
    processor.save_filtered_transactions("completed_transactions.csv", "completed")?;
    
    let customer_summary = processor.get_customer_summary(42);
    println!("Customer 42 has {} transactions totaling {:.2}", 
             customer_summary.0, customer_summary.1);
    
    Ok(())
}