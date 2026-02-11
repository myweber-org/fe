use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WeatherError {
    #[error("Network request failed: {0}")]
    NetworkError(String),
    #[error("API returned error: {0}")]
    ApiError(String),
    #[error("Invalid response format")]
    ParseError,
    #[error("Cache expired")]
    CacheExpired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherData {
    pub temperature: f64,
    pub humidity: f64,
    pub wind_speed: f64,
    pub conditions: String,
    pub timestamp: SystemTime,
}

pub struct WeatherFetcher {
    api_key: String,
    base_url: String,
    cache: HashMap<String, (WeatherData, SystemTime)>,
    cache_duration: Duration,
}

impl WeatherFetcher {
    pub fn new(api_key: String, base_url: String) -> Self {
        Self {
            api_key,
            base_url,
            cache: HashMap::new(),
            cache_duration: Duration::from_secs(300),
        }
    }

    pub async fn get_weather(&mut self, city: &str) -> Result<WeatherData, WeatherError> {
        if let Some((data, timestamp)) = self.cache.get(city) {
            if timestamp.elapsed().unwrap_or(Duration::MAX) < self.cache_duration {
                return Ok(data.clone());
            }
        }

        let weather_data = self.fetch_from_api(city).await?;
        self.cache.insert(
            city.to_string(),
            (weather_data.clone(), SystemTime::now()),
        );
        
        Ok(weather_data)
    }

    async fn fetch_from_api(&self, city: &str) -> Result<WeatherData, WeatherError> {
        let url = format!(
            "{}/weather?q={}&appid={}&units=metric",
            self.base_url, city, self.api_key
        );

        let response = reqwest::get(&url)
            .await
            .map_err(|e| WeatherError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(WeatherError::ApiError(response.status().to_string()));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|_| WeatherError::ParseError)?;

        let weather_data = WeatherData {
            temperature: json["main"]["temp"]
                .as_f64()
                .ok_or(WeatherError::ParseError)?,
            humidity: json["main"]["humidity"]
                .as_f64()
                .ok_or(WeatherError::ParseError)?,
            wind_speed: json["wind"]["speed"]
                .as_f64()
                .ok_or(WeatherError::ParseError)?,
            conditions: json["weather"][0]["description"]
                .as_str()
                .ok_or(WeatherError::ParseError)?
                .to_string(),
            timestamp: SystemTime::now(),
        };

        Ok(weather_data)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn set_cache_duration(&mut self, duration: Duration) {
        self.cache_duration = duration;
    }
}