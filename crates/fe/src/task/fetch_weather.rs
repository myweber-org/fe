use reqwest;
use serde::Deserialize;
use std::env;
use log::{info, error};

#[derive(Debug, Deserialize)]
struct WeatherResponse {
    main: Main,
    name: String,
}

#[derive(Debug, Deserialize)]
struct Main {
    temp: f64,
    humidity: u8,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    let api_key = env::var("WEATHER_API_KEY")
        .expect("WEATHER_API_KEY environment variable not set");
    
    let city = "London";
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );
    
    info!("Fetching weather data for {}", city);
    
    let response = reqwest::get(&url).await?;
    
    if response.status().is_success() {
        let weather: WeatherResponse = response.json().await?;
        info!("Weather in {}: {:.1}°C, {}% humidity", 
              weather.name, weather.main.temp, weather.main.humidity);
        println!("Current weather in {}: {:.1}°C, {}% humidity", 
                 weather.name, weather.main.temp, weather.main.humidity);
    } else {
        error!("Failed to fetch weather data: {}", response.status());
        eprintln!("Error: Unable to fetch weather data");
    }
    
    Ok(())
}