use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WeatherError {
    #[error("Network request failed: {0}")]
    Network(String),
    #[error("Invalid API response: {0}")]
    Parse(String),
    #[error("Service unavailable")]
    ServiceUnavailable,
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
    pub city: String,
    pub temperature: f64,
    pub humidity: u8,
    pub timestamp: Instant,
}

impl WeatherCache {
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            cache: HashMap::new(),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    pub async fn get_weather(&mut self, city: &str, api_key: &str) -> Result<WeatherData, WeatherError> {
        let now = Instant::now();
        
        if let Some((data, timestamp)) = self.cache.get(city) {
            if now.duration_since(*timestamp) < self.ttl {
                return Ok(data.clone());
            }
        }

        let weather = Self::fetch_from_api(city, api_key).await?;
        let data = WeatherData {
            city: city.to_string(),
            temperature: weather.main.temp,
            humidity: weather.main.humidity,
            timestamp: now,
        };
        
        self.cache.insert(city.to_string(), (data.clone(), now));
        Ok(data)
    }

    async fn fetch_from_api(city: &str, api_key: &str) -> Result<WeatherResponse, WeatherError> {
        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
            city, api_key
        );

        let response = reqwest::get(&url)
            .await
            .map_err(|e| WeatherError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(WeatherError::ServiceUnavailable);
        }

        response
            .json::<WeatherResponse>()
            .await
            .map_err(|e| WeatherError::Parse(e.to_string()))
    }

    pub fn clear_expired(&mut self) {
        let now = Instant::now();
        self.cache.retain(|_, (_, timestamp)| {
            now.duration_since(*timestamp) < self.ttl
        });
    }

    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}