use std::error::Error;

#[derive(Debug)]
struct WeatherData {
    city: String,
    temperature: f64,
    condition: String,
    humidity: u8,
}

fn fetch_mock_weather(city: &str) -> Result<WeatherData, Box<dyn Error>> {
    // Simulate a network delay
    std::thread::sleep(std::time::Duration::from_millis(100));

    let mock_data = [
        ("London", 15.5, "Cloudy", 78),
        ("Tokyo", 22.0, "Sunny", 65),
        ("New York", 18.0, "Rainy", 85),
        ("Paris", 17.2, "Partly Cloudy", 70),
        ("Berlin", 16.8, "Clear", 72),
    ];

    for &(mock_city, temp, cond, hum) in &mock_data {
        if mock_city.eq_ignore_ascii_case(city) {
            return Ok(WeatherData {
                city: mock_city.to_string(),
                temperature: temp,
                condition: cond.to_string(),
                humidity: hum,
            });
        }
    }

    Err(format!("Weather data not found for city: {}", city).into())
}

fn display_weather(data: &WeatherData) {
    println!("Weather Report for {}", data.city);
    println!("  Temperature: {:.1}°C", data.temperature);
    println!("  Condition: {}", data.condition);
    println!("  Humidity: {}%", data.humidity);
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <city_name>", args[0]);
        std::process::exit(1);
    }

    let city = &args[1];
    match fetch_mock_weather(city) {
        Ok(weather) => {
            display_weather(&weather);
            Ok(())
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}