
use std::error::Error;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct WeatherData {
    city: String,
    temperature: f64,
    condition: String,
}

fn fetch_weather(city: &str) -> Result<WeatherData, Box<dyn Error>> {
    let mock_response = format!(
        r#"{{
            "city": "{}",
            "temperature": 22.5,
            "condition": "Sunny"
        }}"#,
        city
    );

    let weather: WeatherData = serde_json::from_str(&mock_response)?;
    Ok(weather)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <city_name>", args[0]);
        std::process::exit(1);
    }

    let city = &args[1];
    match fetch_weather(city) {
        Ok(data) => {
            println!("Weather in {}:", data.city);
            println!("  Temperature: {:.1}°C", data.temperature);
            println!("  Condition: {}", data.condition);
        }
        Err(e) => eprintln!("Failed to fetch weather data: {}", e),
    }
}