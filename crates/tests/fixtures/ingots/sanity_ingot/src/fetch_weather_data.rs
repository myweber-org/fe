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