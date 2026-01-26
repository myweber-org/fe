use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use serde::Deserialize;
use reqwest::Error as ReqwestError;

#[derive(Debug, Deserialize)]
struct WeatherResponse {
    main: MainData,
    name: String,
}

#[derive(Debug, Deserialize)]
struct MainData {
    temp: f64,
    humidity: u8,
}

struct WeatherCache {
    data: HashMap<String, (WeatherResponse, SystemTime)>,
    ttl: Duration,
}

impl WeatherCache {
    fn new(ttl_seconds: u64) -> Self {
        WeatherCache {
            data: HashMap::new(),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    async fn get_weather(&mut self, city: &str, api_key: &str) -> Result<WeatherResponse, String> {
        let now = SystemTime::now();
        
        if let Some((cached_data, timestamp)) = self.data.get(city) {
            if now.duration_since(*timestamp).unwrap_or(self.ttl) < self.ttl {
                return Ok(cached_data.clone());
            }
        }

        match self.fetch_weather_from_api(city, api_key).await {
            Ok(weather_data) => {
                self.data.insert(city.to_string(), (weather_data.clone(), now));
                Ok(weather_data)
            }
            Err(e) => Err(format!("Failed to fetch weather: {}", e)),
        }
    }

    async fn fetch_weather_from_api(&self, city: &str, api_key: &str) -> Result<WeatherResponse, ReqwestError> {
        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
            city, api_key
        );
        
        let response = reqwest::get(&url).await?;
        response.json::<WeatherResponse>().await
    }
}

#[tokio::main]
async fn main() {
    let api_key = std::env::var("WEATHER_API_KEY").unwrap_or_else(|_| "demo_key".to_string());
    let mut cache = WeatherCache::new(300);
    
    match cache.get_weather("London", &api_key).await {
        Ok(weather) => {
            println!("Weather in {}: {:.1}°C, {}% humidity", 
                    weather.name, weather.main.temp, weather.main.humidity);
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
use reqwest;
use serde::Deserialize;
use std::error::Error;

const API_KEY: &str = "YOUR_API_KEY_HERE";
const BASE_URL: &str = "https://api.openweathermap.org/data/2.5/weather";

#[derive(Debug, Deserialize)]
pub struct WeatherData {
    name: String,
    main: MainData,
    weather: Vec<WeatherInfo>,
}

#[derive(Debug, Deserialize)]
pub struct MainData {
    temp: f64,
    feels_like: f64,
    humidity: u8,
}

#[derive(Debug, Deserialize)]
pub struct WeatherInfo {
    description: String,
}

pub async fn get_weather(city: &str) -> Result<WeatherData, Box<dyn Error>> {
    let url = format!("{}?q={}&appid={}&units=metric", BASE_URL, city, API_KEY);
    let response = reqwest::get(&url).await?;
    
    if response.status().is_success() {
        let weather_data: WeatherData = response.json().await?;
        Ok(weather_data)
    } else {
        Err(format!("Failed to fetch weather data: {}", response.status()).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_weather_fetch() {
        let result = get_weather("London").await;
        assert!(result.is_ok());
        
        if let Ok(data) = result {
            assert!(!data.name.is_empty());
            assert!(!data.weather.is_empty());
        }
    }
}