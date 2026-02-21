
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WeatherError {
    #[error("Network request failed: {0}")]
    NetworkError(String),
    #[error("API response parsing failed: {0}")]
    ParseError(String),
    #[error("Invalid API key")]
    InvalidApiKey,
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherData {
    pub temperature: f64,
    pub humidity: u8,
    pub wind_speed: f64,
    pub description: String,
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    data: WeatherData,
    expires_at: SystemTime,
}

pub struct WeatherFetcher {
    api_key: String,
    base_url: String,
    cache: HashMap<String, CacheEntry>,
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

    pub fn set_cache_duration(&mut self, duration: Duration) {
        self.cache_duration = duration;
    }

    pub async fn fetch_weather(&mut self, city: &str) -> Result<WeatherData, WeatherError> {
        let cache_key = city.to_lowercase();
        
        if let Some(entry) = self.cache.get(&cache_key) {
            if SystemTime::now() < entry.expires_at {
                return Ok(entry.data.clone());
            }
        }

        let weather_data = self.fetch_from_api(city).await?;
        
        let cache_entry = CacheEntry {
            data: weather_data.clone(),
            expires_at: SystemTime::now() + self.cache_duration,
        };
        
        self.cache.insert(cache_key, cache_entry);
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

        if response.status() == 401 {
            return Err(WeatherError::InvalidApiKey);
        }

        if response.status() == 429 {
            return Err(WeatherError::RateLimitExceeded);
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| WeatherError::ParseError(e.to_string()))?;

        let main_data = json.get("main").ok_or_else(|| {
            WeatherError::ParseError("Missing 'main' field in response".to_string())
        })?;

        let wind_data = json.get("wind").ok_or_else(|| {
            WeatherError::ParseError("Missing 'wind' field in response".to_string())
        })?;

        let weather_array = json.get("weather").and_then(|w| w.as_array()).ok_or_else(|| {
            WeatherError::ParseError("Missing 'weather' array in response".to_string())
        })?;

        let description = weather_array
            .first()
            .and_then(|w| w.get("description"))
            .and_then(|d| d.as_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(WeatherData {
            temperature: main_data["temp"]
                .as_f64()
                .ok_or_else(|| WeatherError::ParseError("Invalid temperature".to_string()))?,
            humidity: main_data["humidity"]
                .as_u64()
                .ok_or_else(|| WeatherError::ParseError("Invalid humidity".to_string()))?
                as u8,
            wind_speed: wind_data["speed"]
                .as_f64()
                .ok_or_else(|| WeatherError::ParseError("Invalid wind speed".to_string()))?,
            description,
            timestamp: SystemTime::now(),
        })
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn remove_expired(&mut self) {
        let now = SystemTime::now();
        self.cache.retain(|_, entry| entry.expires_at > now);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, server_url};

    #[tokio::test]
    async fn test_fetch_weather_success() {
        let _m = mock("GET", "/weather?q=London&appid=test_key&units=metric")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "main": {"temp": 15.5, "humidity": 65},
                "wind": {"speed": 5.2},
                "weather": [{"description": "clear sky"}]
            }"#)
            .create();

        let mut fetcher = WeatherFetcher::new(
            "test_key".to_string(),
            server_url(),
        );

        let result = fetcher.fetch_weather("London").await;
        assert!(result.is_ok());
        
        let weather = result.unwrap();
        assert_eq!(weather.temperature, 15.5);
        assert_eq!(weather.humidity, 65);
        assert_eq!(weather.wind_speed, 5.2);
        assert_eq!(weather.description, "clear sky");
    }

    #[tokio::test]
    async fn test_cache_functionality() {
        let _m = mock("GET", "/weather?q=Paris&appid=test_key&units=metric")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "main": {"temp": 20.0, "humidity": 50},
                "wind": {"speed": 3.0},
                "weather": [{"description": "sunny"}]
            }"#)
            .expect(1)
            .create();

        let mut fetcher = WeatherFetcher::new(
            "test_key".to_string(),
            server_url(),
        );

        let first = fetcher.fetch_weather("Paris").await.unwrap();
        let second = fetcher.fetch_weather("Paris").await.unwrap();
        
        assert_eq!(first.temperature, second.temperature);
        assert_eq!(first.humidity, second.humidity);
    }
}