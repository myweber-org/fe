fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    (celsius * 9.0 / 5.0) + 32.0
}

fn main() {
    let celsius_temperature = 25.0;
    let fahrenheit_temperature = celsius_to_fahrenheit(celsius_temperature);
    println!("{}°C is equal to {}°F", celsius_temperature, fahrenheit_temperature);
}