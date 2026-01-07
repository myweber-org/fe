use reqwest;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WeatherError {
    #[error("Network request failed: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("Invalid API response")]
    InvalidResponse,
    #[error("Cache expired")]
    CacheExpired,
}

#[derive(Debug, Deserialize)]
struct WeatherResponse {
    main: MainData,
    weather: Vec<WeatherInfo>,
    name: String,
}

#[derive(Debug, Deserialize)]
struct MainData {
    temp: f64,
    humidity: u8,
    pressure: u16,
}

#[derive(Debug, Deserialize)]
struct WeatherInfo {
    description: String,
    icon: String,
}

pub struct WeatherCache {
    data: HashMap<String, (WeatherResponse, SystemTime)>,
    ttl: Duration,
}

impl WeatherCache {
    pub fn new(ttl_seconds: u64) -> Self {
        WeatherCache {
            data: HashMap::new(),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    pub fn get(&self, city: &str) -> Option<&WeatherResponse> {
        self.data.get(city).and_then(|(response, timestamp)| {
            if timestamp.elapsed().unwrap_or(self.ttl) < self.ttl {
                Some(response)
            } else {
                None
            }
        })
    }

    pub fn insert(&mut self, city: String, response: WeatherResponse) {
        self.data.insert(city, (response, SystemTime::now()));
    }
}

pub struct WeatherFetcher {
    api_key: String,
    base_url: String,
    cache: WeatherCache,
}

impl WeatherFetcher {
    pub fn new(api_key: String) -> Self {
        WeatherFetcher {
            api_key,
            base_url: "https://api.openweathermap.org/data/2.5/weather".to_string(),
            cache: WeatherCache::new(300),
        }
    }

    pub async fn fetch_weather(&mut self, city: &str) -> Result<WeatherResponse, WeatherError> {
        if let Some(cached) = self.cache.get(city) {
            return Ok(cached.clone());
        }

        let url = format!(
            "{}?q={}&appid={}&units=metric",
            self.base_url, city, self.api_key
        );

        let response = reqwest::get(&url).await?;
        
        if !response.status().is_success() {
            return Err(WeatherError::InvalidResponse);
        }

        let weather_data: WeatherResponse = response.json().await?;
        self.cache.insert(city.to_string(), weather_data.clone());

        Ok(weather_data)
    }

    pub fn display_weather(weather: &WeatherResponse) -> String {
        format!(
            "Weather in {}: {:.1}°C, {}% humidity, {} hPa. Conditions: {}",
            weather.name,
            weather.main.temp,
            weather.main.humidity,
            weather.main.pressure,
            weather.weather.first().map_or("Unknown", |w| &w.description)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, server_url};

    #[tokio::test]
    async fn test_weather_fetch() {
        let _m = mock("GET", "/data/2.5/weather?q=London&appid=test_key&units=metric")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "name": "London",
                "main": {"temp": 15.5, "humidity": 65, "pressure": 1013},
                "weather": [{"description": "clear sky", "icon": "01d"}]
            }"#)
            .create();

        let mut fetcher = WeatherFetcher {
            api_key: "test_key".to_string(),
            base_url: format!("{}/data/2.5/weather", server_url()),
            cache: WeatherCache::new(300),
        };

        let result = fetcher.fetch_weather("London").await;
        assert!(result.is_ok());
        
        let weather = result.unwrap();
        assert_eq!(weather.name, "London");
        assert!((weather.main.temp - 15.5).abs() < 0.1);
    }
}