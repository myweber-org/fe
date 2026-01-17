use std::io;

fn main() {
    println!("Enter temperature in Fahrenheit:");

    let mut fahrenheit = String::new();

    io::stdin()
        .read_line(&mut fahrenheit)
        .expect("Failed to read line");

    let fahrenheit: f64 = fahrenheit.trim().parse().expect("Please type a number!");

    let celsius = (fahrenheit - 32.0) * 5.0 / 9.0;

    println!("{:.2}°F is equal to {:.2}°C", fahrenheit, celsius);
}