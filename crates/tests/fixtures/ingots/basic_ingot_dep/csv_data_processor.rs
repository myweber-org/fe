use csv::{Reader, Writer};
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
    let mut rdr = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in rdr.deserialize() {
        let record: Record = result?;
        records.push(record);
    }

    Ok(records)
}

fn filter_active_records(records: &[Record]) -> Vec<&Record> {
    records.iter().filter(|r| r.active).collect()
}

fn calculate_category_averages(records: &[Record]) -> Vec<(String, f64)> {
    use std::collections::HashMap;
    
    let mut category_sums: HashMap<String, (f64, usize)> = HashMap::new();
    
    for record in records {
        let entry = category_sums.entry(record.category.clone()).or_insert((0.0, 0));
        entry.0 += record.value;
        entry.1 += 1;
    }
    
    category_sums
        .into_iter()
        .map(|(category, (sum, count))| (category, sum / count as f64))
        .collect()
}

fn save_results_to_csv(
    filtered_records: &[&Record],
    averages: &[(String, f64)],
    output_path: &str,
) -> Result<(), Box<dyn Error>> {
    let mut wtr = Writer::from_path(output_path)?;
    
    wtr.write_record(&["ID", "Name", "Category", "Value", "Active"])?;
    for record in filtered_records {
        wtr.serialize(record)?;
    }
    
    wtr.write_record(&[])?;
    wtr.write_record(&["Category", "Average Value"])?;
    for (category, avg) in averages {
        wtr.serialize((category, avg))?;
    }
    
    wtr.flush()?;
    Ok(())
}

fn process_csv_data(input_file: &str, output_file: &str) -> Result<(), Box<dyn Error>> {
    let records = load_csv(input_file)?;
    let active_records = filter_active_records(&records);
    let category_averages = calculate_category_averages(&records);
    
    println!("Total records loaded: {}", records.len());
    println!("Active records found: {}", active_records.len());
    println!("Categories processed: {}", category_averages.len());
    
    save_results_to_csv(&active_records, &category_averages, output_file)?;
    
    for (category, avg) in &category_averages {
        println!("Category '{}' average: {:.2}", category, avg);
    }
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/processed_results.csv";
    
    match process_csv_data(input_file, output_file) {
        Ok(_) => println!("CSV processing completed successfully"),
        Err(e) => eprintln!("Error processing CSV: {}", e),
    }
    
    Ok(())
}