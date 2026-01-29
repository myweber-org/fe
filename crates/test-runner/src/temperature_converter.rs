use std::io;

fn main() {
    println!("Temperature Converter (Celsius to Fahrenheit)");
    println!("Enter temperature in Celsius:");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    let celsius: f64 = match input.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            println!("Please enter a valid number!");
            return;
        }
    };

    let fahrenheit = (celsius * 9.0 / 5.0) + 32.0;
    println!("{:.2}°C is equal to {:.2}°F", celsius, fahrenheit);
}