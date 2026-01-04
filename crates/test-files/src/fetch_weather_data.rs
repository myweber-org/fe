
use reqwest;
use serde_json::Value;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WeatherError {
    #[error("Network request failed: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("Invalid API response format")]
    InvalidResponse,
    #[error("Location not found")]
    LocationNotFound,
}

pub struct WeatherCache {
    cache: HashMap<String, (Value, Instant)>,
    ttl: Duration,
}

impl WeatherCache {
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            cache: HashMap::new(),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    pub async fn get_weather(
        &mut self,
        api_key: &str,
        location: &str,
    ) -> Result<Value, WeatherError> {
        let cache_key = format!("{}-{}", api_key, location);

        if let Some((cached_data, timestamp)) = self.cache.get(&cache_key) {
            if timestamp.elapsed() < self.ttl {
                return Ok(cached_data.clone());
            }
        }

        let weather_data = self.fetch_weather_from_api(api_key, location).await?;
        self.cache.insert(cache_key, (weather_data.clone(), Instant::now()));

        Ok(weather_data)
    }

    async fn fetch_weather_from_api(
        &self,
        api_key: &str,
        location: &str,
    ) -> Result<Value, WeatherError> {
        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
            location, api_key
        );

        let response = reqwest::get(&url).await?;
        
        if response.status().is_client_error() {
            return Err(WeatherError::LocationNotFound);
        }

        let weather_data: Value = response.json().await?;
        
        if weather_data.get("main").is_none() {
            return Err(WeatherError::InvalidResponse);
        }

        Ok(weather_data)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn remove_expired(&mut self) {
        let now = Instant::now();
        self.cache.retain(|_, (_, timestamp)| now.duration_since(*timestamp) < self.ttl);
    }
}

pub fn extract_temperature(weather_data: &Value) -> Option<f64> {
    weather_data["main"]["temp"].as_f64()
}

pub fn extract_weather_description(weather_data: &Value) -> Option<&str> {
    weather_data["weather"][0]["description"].as_str()
}