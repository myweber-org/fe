use clap::{App, Arg};
use csv::{Reader, Writer};
use std::error::Error;
use std::fs::File;

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("CSV Filter")
        .version("1.0")
        .author("Data Processor")
        .about("Filters CSV rows based on column criteria")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("FILE")
                .help("Input CSV file path")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("FILE")
                .help("Output CSV file path")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("column")
                .short("c")
                .long("column")
                .value_name("COLUMN")
                .help("Column name to filter by")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("value")
                .short("v")
                .long("value")
                .value_name("VALUE")
                .help("Value to match in specified column")
                .required(true)
                .takes_value(true),
        )
        .get_matches();

    let input_path = matches.value_of("input").unwrap();
    let output_path = matches.value_of("output").unwrap();
    let filter_column = matches.value_of("column").unwrap();
    let filter_value = matches.value_of("value").unwrap();

    let input_file = File::open(input_path)?;
    let mut rdr = Reader::from_reader(input_file);
    let output_file = File::create(output_path)?;
    let mut wtr = Writer::from_writer(output_file);

    let headers = rdr.headers()?.clone();
    wtr.write_record(&headers)?;

    let column_index = headers
        .iter()
        .position(|h| h == filter_column)
        .ok_or_else(|| format!("Column '{}' not found", filter_column))?;

    for result in rdr.records() {
        let record = result?;
        if record.get(column_index) == Some(filter_value) {
            wtr.write_record(&record)?;
        }
    }

    wtr.flush()?;
    println!("Filtered data written to {}", output_path);
    Ok(())
}