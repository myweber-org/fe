
use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Transaction {
    id: u32,
    customer_id: u32,
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
        let mut rdr = Reader::from_reader(file);
        
        for result in rdr.deserialize() {
            let transaction: Transaction = result?;
            self.transactions.push(transaction);
        }
        
        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<&Transaction> {
        self.transactions
            .iter()
            .filter(|t| t.category == category)
            .collect()
    }

    fn calculate_total_amount(&self) -> f64 {
        self.transactions.iter().map(|t| t.amount).sum()
    }

    fn calculate_average_amount(&self) -> f64 {
        if self.transactions.is_empty() {
            0.0
        } else {
            self.calculate_total_amount() / self.transactions.len() as f64
        }
    }

    fn find_largest_transaction(&self) -> Option<&Transaction> {
        self.transactions.iter().max_by(|a, b| {
            a.amount.partial_cmp(&b.amount).unwrap()
        })
    }

    fn save_filtered_to_csv(&self, category: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        let mut wtr = Writer::from_path(output_path)?;
        
        for transaction in filtered {
            wtr.serialize(transaction)?;
        }
        
        wtr.flush()?;
        Ok(())
    }

    fn aggregate_by_customer(&self) -> Vec<(u32, f64)> {
        use std::collections::HashMap;
        
        let mut aggregates: HashMap<u32, f64> = HashMap::new();
        
        for transaction in &self.transactions {
            *aggregates.entry(transaction.customer_id).or_insert(0.0) += transaction.amount;
        }
        
        aggregates.into_iter().collect()
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut processor = TransactionProcessor::new();
    
    processor.load_from_csv("transactions.csv")?;
    
    println!("Total transactions: {}", processor.transactions.len());
    println!("Total amount: ${:.2}", processor.calculate_total_amount());
    println!("Average transaction: ${:.2}", processor.calculate_average_amount());
    
    if let Some(largest) = processor.find_largest_transaction() {
        println!("Largest transaction: ID {} - ${:.2}", largest.id, largest.amount);
    }
    
    let electronics_transactions = processor.filter_by_category("Electronics");
    println!("Electronics transactions: {}", electronics_transactions.len());
    
    processor.save_filtered_to_csv("Electronics", "electronics_transactions.csv")?;
    
    let customer_totals = processor.aggregate_by_customer();
    for (customer_id, total) in customer_totals {
        println!("Customer {} total: ${:.2}", customer_id, total);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_empty_processor() {
        let processor = TransactionProcessor::new();
        assert_eq!(processor.transactions.len(), 0);
        assert_eq!(processor.calculate_total_amount(), 0.0);
        assert_eq!(processor.calculate_average_amount(), 0.0);
    }

    #[test]
    fn test_filter_by_category() {
        let mut processor = TransactionProcessor::new();
        processor.transactions.push(Transaction {
            id: 1,
            customer_id: 100,
            amount: 50.0,
            category: "Electronics".to_string(),
            timestamp: "2024-01-01".to_string(),
        });
        
        processor.transactions.push(Transaction {
            id: 2,
            customer_id: 101,
            amount: 30.0,
            category: "Clothing".to_string(),
            timestamp: "2024-01-02".to_string(),
        });
        
        let electronics = processor.filter_by_category("Electronics");
        assert_eq!(electronics.len(), 1);
        assert_eq!(electronics[0].id, 1);
    }

    #[test]
    fn test_csv_operations() -> Result<(), Box<dyn Error>> {
        let csv_data = "id,customer_id,amount,category,timestamp\n1,100,50.0,Electronics,2024-01-01\n2,101,30.0,Clothing,2024-01-02";
        
        let mut temp_file = NamedTempFile::new()?;
        write!(temp_file, "{}", csv_data)?;
        
        let mut processor = TransactionProcessor::new();
        processor.load_from_csv(temp_file.path().to_str().unwrap())?;
        
        assert_eq!(processor.transactions.len(), 2);
        assert_eq!(processor.calculate_total_amount(), 80.0);
        
        Ok(())
    }
}