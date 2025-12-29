use reqwest;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WeatherError {
    #[error("Network request failed: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("API returned error: {0}")]
    ApiError(String),
    #[error("Invalid location provided")]
    InvalidLocation,
    #[error("Cache error")]
    CacheError,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WeatherData {
    temperature: f64,
    humidity: f64,
    description: String,
    wind_speed: f64,
}

pub struct WeatherCache {
    cache: Arc<Mutex<HashMap<String, (WeatherData, SystemTime)>>>,
    ttl: Duration,
}

impl WeatherCache {
    pub fn new(ttl_seconds: u64) -> Self {
        WeatherCache {
            cache: Arc::new(Mutex::new(HashMap::new())),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    pub fn get(&self, location: &str) -> Option<WeatherData> {
        let cache = self.cache.lock().ok()?;
        cache.get(location).and_then(|(data, timestamp)| {
            if timestamp.elapsed().ok()? < self.ttl {
                Some(data.clone())
            } else {
                None
            }
        })
    }

    pub fn set(&self, location: String, data: WeatherData) -> Result<(), WeatherError> {
        let mut cache = self.cache.lock().map_err(|_| WeatherError::CacheError)?;
        cache.insert(location, (data, SystemTime::now()));
        Ok(())
    }
}

pub struct WeatherFetcher {
    api_key: String,
    base_url: String,
    cache: WeatherCache,
    client: reqwest::Client,
}

impl WeatherFetcher {
    pub fn new(api_key: String, cache_ttl: u64) -> Self {
        WeatherFetcher {
            api_key,
            base_url: "https://api.weather.example.com".to_string(),
            cache: WeatherCache::new(cache_ttl),
            client: reqwest::Client::new(),
        }
    }

    pub async fn fetch_weather(&self, location: &str) -> Result<WeatherData, WeatherError> {
        if location.trim().is_empty() {
            return Err(WeatherError::InvalidLocation);
        }

        if let Some(cached) = self.cache.get(location) {
            return Ok(cached);
        }

        let url = format!("{}/weather?location={}&apikey={}", 
            self.base_url, location, self.api_key);
        
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            return Err(WeatherError::ApiError(
                format!("HTTP {}: {}", response.status(), response.text().await?)
            ));
        }

        let weather_data: WeatherData = response.json().await?;
        
        self.cache.set(location.to_string(), weather_data.clone())?;
        
        Ok(weather_data)
    }

    pub async fn fetch_multiple_locations(
        &self, 
        locations: &[&str]
    ) -> Result<Vec<(String, Result<WeatherData, WeatherError>)>, WeatherError> {
        let mut results = Vec::new();
        
        for location in locations {
            let result = self.fetch_weather(location).await;
            results.push((location.to_string(), result));
        }
        
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    #[tokio::test]
    async fn test_fetch_weather_success() {
        let mut server = Server::new_async().await;
        let mock = server.mock("GET", "/weather?location=London&apikey=test_key")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"temperature": 15.5, "humidity": 65.0, "description": "Cloudy", "wind_speed": 12.3}"#)
            .create_async()
            .await;

        let fetcher = WeatherFetcher {
            api_key: "test_key".to_string(),
            base_url: server.url(),
            cache: WeatherCache::new(300),
            client: reqwest::Client::new(),
        };

        let result = fetcher.fetch_weather("London").await;
        assert!(result.is_ok());
        
        let weather = result.unwrap();
        assert_eq!(weather.temperature, 15.5);
        assert_eq!(weather.description, "Cloudy");
        
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_cache_functionality() {
        let fetcher = WeatherFetcher::new("test_key".to_string(), 300);
        
        let location = "Paris";
        let test_data = WeatherData {
            temperature: 20.0,
            humidity: 50.0,
            description: "Sunny".to_string(),
            wind_speed: 5.0,
        };

        fetcher.cache.set(location.to_string(), test_data.clone()).unwrap();
        
        let cached = fetcher.cache.get(location);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().temperature, 20.0);
    }

    #[test]
    fn test_weather_cache_expiry() {
        let cache = WeatherCache::new(1);
        let test_data = WeatherData {
            temperature: 25.0,
            humidity: 60.0,
            description: "Hot".to_string(),
            wind_speed: 3.0,
        };

        cache.set("Tokyo".to_string(), test_data).unwrap();
        
        assert!(cache.get("Tokyo").is_some());
        
        std::thread::sleep(Duration::from_secs(2));
        
        assert!(cache.get("Tokyo").is_none());
    }
}