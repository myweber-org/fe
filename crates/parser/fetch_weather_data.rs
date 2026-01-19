
use std::env;
use std::process;
use reqwest::blocking::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct WeatherData {
    location: String,
    temperature: f64,
    condition: String,
}

fn fetch_weather(api_key: &str, city: &str) -> Result<WeatherData, Box<dyn std::error::Error>> {
    let url = format!("https://api.weather.mock/v1/current?key={}&city={}", api_key, city);
    let client = Client::new();
    let response = client.get(&url).send()?;
    
    if response.status().is_success() {
        let weather: WeatherData = response.json()?;
        Ok(weather)
    } else {
        Err(format!("API request failed with status: {}", response.status()).into())
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <api_key> <city>", args[0]);
        process::exit(1);
    }

    let api_key = &args[1];
    let city = &args[2];

    match fetch_weather(api_key, city) {
        Ok(weather) => {
            println!("Weather in {}:", weather.location);
            println!("  Temperature: {:.1}°C", weather.temperature);
            println!("  Condition: {}", weather.condition);
        }
        Err(e) => {
            eprintln!("Error fetching weather data: {}", e);
            process::exit(1);
        }
    }
}