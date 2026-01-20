fn fahrenheit_to_celsius(f: f64) -> f64 {
    (f - 32.0) * 5.0 / 9.0
}

fn main() {
    let fahrenheit_temp = 77.0;
    let celsius_temp = fahrenheit_to_celsius(fahrenheit_temp);
    println!("{}°F is {:.2}°C", fahrenheit_temp, celsius_temp);
}