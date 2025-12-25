use std::io;

fn main() {
    println!("Temperature Converter");
    println!("1. Celsius to Fahrenheit");
    println!("2. Fahrenheit to Celsius");

    let choice: u32 = loop {
        println!("Please enter your choice (1 or 2):");
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        match input.trim().parse() {
            Ok(num) if num == 1 || num == 2 => break num,
            _ => println!("Invalid input. Please enter 1 or 2."),
        }
    };

    let temperature: f64 = loop {
        println!("Please enter the temperature:");
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        match input.trim().parse() {
            Ok(num) => break num,
            Err(_) => println!("Invalid input. Please enter a number."),
        }
    };

    let result = if choice == 1 {
        celsius_to_fahrenheit(temperature)
    } else {
        fahrenheit_to_celsius(temperature)
    };

    println!("Converted temperature: {:.2}", result);
}

fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    (celsius * 9.0 / 5.0) + 32.0
}

fn fahrenheit_to_celsius(fahrenheit: f64) -> f64 {
    (fahrenheit - 32.0) * 5.0 / 9.0
}