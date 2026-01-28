
fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    (celsius * 9.0 / 5.0) + 32.0
}

fn main() {
    let celsius_temps = [0.0, 20.0, 37.0, 100.0];
    
    for &temp in &celsius_temps {
        let fahrenheit = celsius_to_fahrenheit(temp);
        println!("{:.1}°C = {:.1}°F", temp, fahrenheit);
    }
}