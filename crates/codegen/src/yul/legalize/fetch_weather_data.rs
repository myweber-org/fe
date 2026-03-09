use reqwest;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
struct WeatherData {
    temperature: f64,
    humidity: u8,
    condition: String,
}

async fn fetch_weather_data(api_key: &str, city: &str) -> Result<WeatherData, Box<dyn Error>> {
    let url = format!("https://api.weather.example.com/data?city={}&key={}", city, api_key);
    
    let response = reqwest::get(&url).await?;
    
    if !response.status().is_success() {
        return Err(format!("API request failed with status: {}", response.status()).into());
    }
    
    let weather: WeatherData = response.json().await?;
    
    Ok(weather)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let api_key = "YOUR_API_KEY_HERE";
    let city = "London";
    
    match fetch_weather_data(api_key, city).await {
        Ok(data) => {
            println!("Weather in {}:", city);
            println!("Temperature: {:.1}°C", data.temperature);
            println!("Humidity: {}%", data.humidity);
            println!("Condition: {}", data.condition);
        }
        Err(e) => eprintln!("Failed to fetch weather data: {}", e),
    }
    
    Ok(())
}