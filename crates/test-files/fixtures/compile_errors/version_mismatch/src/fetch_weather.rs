use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WeatherError {
    #[error("Network request failed: {0}")]
    RequestFailed(String),
    #[error("Invalid API response: {0}")]
    InvalidResponse(String),
    #[error("Cache expired")]
    CacheExpired,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WeatherData {
    pub temperature: f64,
    pub humidity: f64,
    pub description: String,
    pub timestamp: SystemTime,
}

pub struct WeatherFetcher {
    api_key: String,
    base_url: String,
    cache: HashMap<String, (WeatherData, SystemTime)>,
    cache_duration: Duration,
}

impl WeatherFetcher {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.weather.example.com".to_string(),
            cache: HashMap::new(),
            cache_duration: Duration::from_secs(300),
        }
    }

    pub async fn get_weather(&mut self, city: &str) -> Result<WeatherData, WeatherError> {
        let cache_key = city.to_lowercase();
        
        if let Some((data, cached_at)) = self.cache.get(&cache_key) {
            if cached_at.elapsed().unwrap_or(Duration::MAX) < self.cache_duration {
                return Ok(data.clone());
            }
        }

        let weather_data = self.fetch_from_api(city).await?;
        self.cache.insert(cache_key, (weather_data.clone(), SystemTime::now()));
        
        Ok(weather_data)
    }

    async fn fetch_from_api(&self, city: &str) -> Result<WeatherData, WeatherError> {
        let url = format!("{}/weather?city={}&api_key={}", self.base_url, city, self.api_key);
        
        let response = reqwest::get(&url)
            .await
            .map_err(|e| WeatherError::RequestFailed(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(WeatherError::RequestFailed(format!("HTTP {}", response.status())));
        }

        let json: serde_json::Value = response.json()
            .await
            .map_err(|e| WeatherError::InvalidResponse(e.to_string()))?;

        let weather = WeatherData {
            temperature: json["main"]["temp"]
                .as_f64()
                .ok_or_else(|| WeatherError::InvalidResponse("Missing temperature".to_string()))?,
            humidity: json["main"]["humidity"]
                .as_f64()
                .ok_or_else(|| WeatherError::InvalidResponse("Missing humidity".to_string()))?,
            description: json["weather"][0]["description"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
            timestamp: SystemTime::now(),
        };

        Ok(weather)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn set_cache_duration(&mut self, duration: Duration) {
        self.cache_duration = duration;
    }
}