
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use serde::{Deserialize, Serialize};
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Transaction {
    id: u32,
    customer_id: String,
    amount: f64,
    category: String,
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

    fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);
        
        for result in csv_reader.deserialize() {
            let transaction: Transaction = result?;
            self.transactions.push(transaction);
        }
        
        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<Transaction> {
        self.transactions
            .iter()
            .filter(|t| t.category == category)
            .cloned()
            .collect()
    }

    fn calculate_total_amount(&self) -> f64 {
        self.transactions.iter().map(|t| t.amount).sum()
    }

    fn calculate_average_amount(&self) -> f64 {
        if self.transactions.is_empty() {
            return 0.0;
        }
        self.calculate_total_amount() / self.transactions.len() as f64
    }

    fn get_customer_summary(&self) -> Vec<(String, f64)> {
        let mut summary: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
        
        for transaction in &self.transactions {
            *summary.entry(transaction.customer_id.clone()).or_insert(0.0) += transaction.amount;
        }
        
        let mut result: Vec<(String, f64)> = summary.into_iter().collect();
        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        result
    }

    fn save_filtered_to_csv(&self, category: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        
        let file = File::create(output_path)?;
        let writer = BufWriter::new(file);
        let mut csv_writer = WriterBuilder::new().has_headers(true).from_writer(writer);
        
        for transaction in filtered {
            csv_writer.serialize(transaction)?;
        }
        
        csv_writer.flush()?;
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut processor = TransactionProcessor::new();
    
    match processor.load_from_csv("transactions.csv") {
        Ok(_) => println!("Successfully loaded transactions"),
        Err(e) => eprintln!("Error loading CSV: {}", e),
    }
    
    let electronics_transactions = processor.filter_by_category("Electronics");
    println!("Found {} electronics transactions", electronics_transactions.len());
    
    let total = processor.calculate_total_amount();
    println!("Total transaction amount: ${:.2}", total);
    
    let average = processor.calculate_average_amount();
    println!("Average transaction amount: ${:.2}", average);
    
    let customer_summary = processor.get_customer_summary();
    println!("Top customers by spending:");
    for (customer, amount) in customer_summary.iter().take(5) {
        println!("  {}: ${:.2}", customer, amount);
    }
    
    processor.save_filtered_to_csv("Electronics", "electronics_transactions.csv")?;
    println!("Filtered transactions saved to electronics_transactions.csv");
    
    Ok(())
}