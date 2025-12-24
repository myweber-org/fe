use std::io;

fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    (celsius * 9.0 / 5.0) + 32.0
}

fn fahrenheit_to_celsius(fahrenheit: f64) -> f64 {
    (fahrenheit - 32.0) * 5.0 / 9.0
}

fn main() {
    println!("Temperature Converter");
    println!("1. Celsius to Fahrenheit");
    println!("2. Fahrenheit to Celsius");

    let mut choice = String::new();
    io::stdin()
        .read_line(&mut choice)
        .expect("Failed to read line");

    let choice: u32 = match choice.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            println!("Invalid input. Please enter 1 or 2.");
            return;
        }
    };

    println!("Enter temperature value:");
    let mut temp_input = String::new();
    io::stdin()
        .read_line(&mut temp_input)
        .expect("Failed to read line");

    let temperature: f64 = match temp_input.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            println!("Invalid temperature value.");
            return;
        }
    };

    match choice {
        1 => {
            let result = celsius_to_fahrenheit(temperature);
            println!("{:.2}°C = {:.2}°F", temperature, result);
        }
        2 => {
            let result = fahrenheit_to_celsius(temperature);
            println!("{:.2}°F = {:.2}°C", temperature, result);
        }
        _ => println!("Invalid choice. Please select 1 or 2."),
    }
}