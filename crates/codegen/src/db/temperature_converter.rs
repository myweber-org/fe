fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    (celsius * 9.0 / 5.0) + 32.0
}

fn celsius_to_kelvin(celsius: f64) -> f64 {
    celsius + 273.15
}

fn main() {
    let celsius_temp = 25.0;
    let fahrenheit_temp = celsius_to_fahrenheit(celsius_temp);
    let kelvin_temp = celsius_to_kelvin(celsius_temp);
    
    println!("Temperature Conversions:");
    println!("{:.2}°C = {:.2}°F", celsius_temp, fahrenheit_temp);
    println!("{:.2}°C = {:.2}K", celsius_temp, kelvin_temp);
}