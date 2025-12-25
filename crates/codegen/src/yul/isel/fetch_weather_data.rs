use std::error::Error;
use reqwest;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct WeatherData {
    temperature: f64,
    humidity: u8,
    condition: String,
}

async fn get_weather(city: &str) -> Result<WeatherData, Box<dyn Error>> {
    let url = format!("https://api.mockweather.example.com/current?city={}", city);
    let response = reqwest::get(&url).await?;
    
    if response.status().is_success() {
        let weather: WeatherData = response.json().await?;
        Ok(weather)
    } else {
        Err("Failed to fetch weather data".into())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <city_name>", args[0]);
        std::process::exit(1);
    }
    
    let city = &args[1];
    match get_weather(city).await {
        Ok(data) => {
            println!("Weather in {}:", city);
            println!("  Temperature: {:.1}°C", data.temperature);
            println!("  Humidity: {}%", data.humidity);
            println!("  Condition: {}", data.condition);
        }
        Err(e) => eprintln!("Error: {}", e),
    }
    
    Ok(())
}