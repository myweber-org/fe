use reqwest;
use serde::Deserialize;
use std::error::Error;

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

async fn fetch_weather_data(api_key: &str, city: &str) -> Result<WeatherResponse, Box<dyn Error>> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );
    
    let response = reqwest::get(&url).await?;
    
    if !response.status().is_success() {
        return Err(format!("API request failed with status: {}", response.status()).into());
    }
    
    let weather_data: WeatherResponse = response.json().await?;
    Ok(weather_data)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let api_key = "your_api_key_here";
    let city = "London";
    
    match fetch_weather_data(api_key, city).await {
        Ok(data) => {
            println!("Weather in {}:", data.name);
            println!("Temperature: {:.1}°C", data.main.temp);
            println!("Humidity: {}%", data.main.humidity);
        }
        Err(e) => eprintln!("Failed to fetch weather data: {}", e),
    }
    
    Ok(())
}
use reqwest;
use serde::Deserialize;
use std::error::Error;

#[derive(Deserialize, Debug)]
struct WeatherData {
    name: String,
    main: Main,
    weather: Vec<Weather>,
}

#[derive(Deserialize, Debug)]
struct Main {
    temp: f64,
    humidity: u8,
}

#[derive(Deserialize, Debug)]
struct Weather {
    description: String,
}

pub async fn get_weather(city: &str, api_key: &str) -> Result<WeatherData, Box<dyn Error>> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );
    
    let response = reqwest::get(&url).await?;
    
    if !response.status().is_success() {
        return Err(format!("API request failed with status: {}", response.status()).into());
    }
    
    let weather_data: WeatherData = response.json().await?;
    Ok(weather_data)
}

pub fn display_weather(data: &WeatherData) {
    println!("Weather in {}:", data.name);
    println!("Temperature: {:.1}°C", data.main.temp);
    println!("Humidity: {}%", data.main.humidity);
    
    if let Some(weather) = data.weather.first() {
        println!("Conditions: {}", weather.description);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, server_url};

    #[tokio::test]
    async fn test_get_weather_success() {
        let mock = mock("GET", "/data/2.5/weather?q=London&appid=test_key&units=metric")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "name": "London",
                "main": {"temp": 15.5, "humidity": 65},
                "weather": [{"description": "clear sky"}]
            }"#)
            .create();

        let _guard = mockito::server_guard();
        let url = server_url();
        
        let result = get_weather("London", "test_key").await;
        assert!(result.is_ok());
        
        let data = result.unwrap();
        assert_eq!(data.name, "London");
        assert_eq!(data.main.temp, 15.5);
        assert_eq!(data.main.humidity, 65);
        assert_eq!(data.weather[0].description, "clear sky");
    }
}