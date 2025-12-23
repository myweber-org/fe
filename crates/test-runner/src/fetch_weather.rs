use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WeatherError {
    #[error("Network request failed: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Invalid API response: {0}")]
    Parse(String),
    #[error("Location not found")]
    LocationNotFound,
}

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

pub struct WeatherCache {
    cache: HashMap<String, (WeatherData, Instant)>,
    ttl: Duration,
}

#[derive(Debug, Clone)]
pub struct WeatherData {
    pub location: String,
    pub temperature: f64,
    pub humidity: u8,
}

impl WeatherCache {
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            cache: HashMap::new(),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    pub async fn get_weather(&mut self, location: &str, api_key: &str) -> Result<WeatherData, WeatherError> {
        let key = location.to_lowercase();
        
        if let Some((data, timestamp)) = self.cache.get(&key) {
            if timestamp.elapsed() < self.ttl {
                return Ok(data.clone());
            }
        }

        let weather = Self::fetch_from_api(location, api_key).await?;
        self.cache.insert(key, (weather.clone(), Instant::now()));
        
        Ok(weather)
    }

    async fn fetch_from_api(location: &str, api_key: &str) -> Result<WeatherData, WeatherError> {
        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
            location, api_key
        );

        let response: WeatherResponse = reqwest::get(&url)
            .await?
            .json()
            .await
            .map_err(|e| WeatherError::Parse(e.to_string()))?;

        Ok(WeatherData {
            location: response.name,
            temperature: response.main.temp,
            humidity: response.main.humidity,
        })
    }

    pub fn clear_expired(&mut self) {
        let now = Instant::now();
        self.cache.retain(|_, (_, timestamp)| now.duration_since(*timestamp) < self.ttl);
    }

    pub fn clear_all(&mut self) {
        self.cache.clear();
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
            .with_body(r#"{"name":"London","main":{"temp":15.5,"humidity":65}}"#)
            .create();

        let mut cache = WeatherCache::new(300);
        let result = cache.get_weather("London", "test_key").await;
        
        assert!(result.is_ok());
        let weather = result.unwrap();
        assert_eq!(weather.location, "London");
        assert_eq!(weather.temperature, 15.5);
        assert_eq!(weather.humidity, 65);
    }
}