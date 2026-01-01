fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    (celsius * 9.0 / 5.0) + 32.0
}

fn fahrenheit_to_celsius(fahrenheit: f64) -> f64 {
    (fahrenheit - 32.0) * 5.0 / 9.0
}

fn main() {
    let celsius_temp = 25.0;
    let fahrenheit_temp = celsius_to_fahrenheit(celsius_temp);
    println!("{:.2}°C is {:.2}°F", celsius_temp, fahrenheit_temp);

    let converted_back = fahrenheit_to_celsius(fahrenheit_temp);
    println!("{:.2}°F is {:.2}°C", fahrenheit_temp, converted_back);
}