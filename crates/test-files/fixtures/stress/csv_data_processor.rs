
use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Transaction {
    id: u32,
    customer: String,
    amount: f64,
    category: String,
    timestamp: String,
}

fn load_transactions(file_path: &str) -> Result<Vec<Transaction>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);
    let mut transactions = Vec::new();

    for result in reader.deserialize() {
        let record: Transaction = result?;
        transactions.push(record);
    }

    Ok(transactions)
}

fn filter_by_category(transactions: &[Transaction], category: &str) -> Vec<Transaction> {
    transactions
        .iter()
        .filter(|t| t.category == category)
        .cloned()
        .collect()
}

fn calculate_total_amount(transactions: &[Transaction]) -> f64 {
    transactions.iter().map(|t| t.amount).sum()
}

fn aggregate_by_customer(transactions: &[Transaction]) -> Vec<(String, f64)> {
    use std::collections::HashMap;

    let mut aggregates = HashMap::new();
    for transaction in transactions {
        *aggregates.entry(transaction.customer.clone()).or_insert(0.0) += transaction.amount;
    }

    aggregates.into_iter().collect()
}

fn save_aggregated_data(
    aggregated: &[(String, f64)],
    output_path: &str,
) -> Result<(), Box<dyn Error>> {
    let file = File::create(output_path)?;
    let mut writer = Writer::from_writer(file);

    for (customer, total) in aggregated {
        writer.serialize((customer, total))?;
    }

    writer.flush()?;
    Ok(())
}

fn process_transaction_data(
    input_file: &str,
    category_filter: Option<&str>,
    output_file: &str,
) -> Result<(), Box<dyn Error>> {
    let transactions = load_transactions(input_file)?;

    let filtered_transactions = if let Some(category) = category_filter {
        filter_by_category(&transactions, category)
    } else {
        transactions.clone()
    };

    let total = calculate_total_amount(&filtered_transactions);
    println!("Total amount: {:.2}", total);

    let aggregated = aggregate_by_customer(&filtered_transactions);
    save_aggregated_data(&aggregated, output_file)?;

    println!("Processed {} transactions", filtered_transactions.len());
    println!("Aggregated data saved to {}", output_file);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_filter_by_category() {
        let transactions = vec![
            Transaction {
                id: 1,
                customer: "Alice".to_string(),
                amount: 100.0,
                category: "Food".to_string(),
                timestamp: "2023-01-01".to_string(),
            },
            Transaction {
                id: 2,
                customer: "Bob".to_string(),
                amount: 200.0,
                category: "Electronics".to_string(),
                timestamp: "2023-01-02".to_string(),
            },
        ];

        let filtered = filter_by_category(&transactions, "Food");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].customer, "Alice");
    }

    #[test]
    fn test_calculate_total_amount() {
        let transactions = vec![
            Transaction {
                id: 1,
                customer: "Alice".to_string(),
                amount: 100.0,
                category: "Food".to_string(),
                timestamp: "2023-01-01".to_string(),
            },
            Transaction {
                id: 2,
                customer: "Bob".to_string(),
                amount: 200.0,
                category: "Electronics".to_string(),
                timestamp: "2023-01-02".to_string(),
            },
        ];

        let total = calculate_total_amount(&transactions);
        assert_eq!(total, 300.0);
    }
}