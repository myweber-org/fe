use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

fn filter_records_by_category(
    input_path: &Path,
    output_path: &Path,
    target_category: &str,
) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut writer = Writer::from_path(output_path)?;

    for result in reader.deserialize() {
        let record: Record = result?;
        if record.category == target_category && record.active {
            writer.serialize(&record)?;
        }
    }

    writer.flush()?;
    Ok(())
}

fn calculate_category_average(input_path: &Path) -> Result<Vec<(String, f64)>, Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut category_totals: std::collections::HashMap<String, (f64, u32)> =
        std::collections::HashMap::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        if record.active {
            let entry = category_totals
                .entry(record.category.clone())
                .or_insert((0.0, 0));
            entry.0 += record.value;
            entry.1 += 1;
        }
    }

    let mut averages: Vec<(String, f64)> = category_totals
        .into_iter()
        .map(|(category, (total, count))| (category, total / count as f64))
        .collect();
    averages.sort_by(|a, b| a.0.cmp(&b.0));

    Ok(averages)
}

fn generate_sample_data(output_path: &Path) -> Result<(), Box<dyn Error>> {
    let mut writer = Writer::from_path(output_path)?;
    let sample_records = vec![
        Record {
            id: 1,
            name: String::from("Item Alpha"),
            category: String::from("Electronics"),
            value: 299.99,
            active: true,
        },
        Record {
            id: 2,
            name: String::from("Item Beta"),
            category: String::from("Books"),
            value: 24.50,
            active: true,
        },
        Record {
            id: 3,
            name: String::from("Item Gamma"),
            category: String::from("Electronics"),
            value: 450.00,
            active: false,
        },
        Record {
            id: 4,
            name: String::from("Item Delta"),
            category: String::from("Books"),
            value: 18.75,
            active: true,
        },
        Record {
            id: 5,
            name: String::from("Item Epsilon"),
            category: String::from("Clothing"),
            value: 65.30,
            active: true,
        },
    ];

    for record in sample_records {
        writer.serialize(&record)?;
    }

    writer.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let sample_path = Path::new("sample_data.csv");
    let filtered_path = Path::new("filtered_electronics.csv");

    println!("Generating sample CSV data...");
    generate_sample_data(sample_path)?;

    println!("Filtering active electronics records...");
    filter_records_by_category(sample_path, filtered_path, "Electronics")?;

    println!("Calculating category averages...");
    let averages = calculate_category_average(sample_path)?;

    for (category, avg) in averages {
        println!("Category: {}, Average Value: {:.2}", category, avg);
    }

    Ok(())
}