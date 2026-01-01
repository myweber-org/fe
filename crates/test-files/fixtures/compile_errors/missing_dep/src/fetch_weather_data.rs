use reqwest;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WeatherError {
    #[error("Network request failed: {0}")]
    Network(#[from] reqwest::Error),
    #[error("API response error: {0}")]
    Api(String),
    #[error("Invalid location format")]
    InvalidLocation,
}

#[derive(Deserialize, Debug)]
struct ApiResponse {
    main: MainData,
    weather: Vec<WeatherCondition>,
    name: String,
}

#[derive(Deserialize, Debug)]
struct MainData {
    temp: f64,
    humidity: u8,
    pressure: u16,
}

#[derive(Deserialize, Debug)]
struct WeatherCondition {
    description: String,
    icon: String,
}

pub struct WeatherCache {
    entries: HashMap<String, (SystemTime, WeatherData)>,
    ttl: Duration,
}

#[derive(Debug, Clone)]
pub struct WeatherData {
    pub location: String,
    pub temperature: f64,
    pub humidity: u8,
    pub pressure: u16,
    pub description: String,
    pub icon_code: String,
}

impl WeatherCache {
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            entries: HashMap::new(),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    pub fn get(&self, location: &str) -> Option<WeatherData> {
        self.entries.get(location).and_then(|(timestamp, data)| {
            if timestamp.elapsed().unwrap_or(self.ttl) < self.ttl {
                Some(data.clone())
            } else {
                None
            }
        })
    }

    pub fn insert(&mut self, location: String, data: WeatherData) {
        self.entries.insert(location, (SystemTime::now(), data));
    }

    pub fn clear_expired(&mut self) {
        let now = SystemTime::now();
        self.entries.retain(|_, (timestamp, _)| {
            now.duration_since(*timestamp).unwrap_or(self.ttl) < self.ttl
        });
    }
}

pub struct WeatherFetcher {
    client: reqwest::Client,
    api_key: String,
    cache: WeatherCache,
}

impl WeatherFetcher {
    pub fn new(api_key: String, cache_ttl_seconds: u64) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            cache: WeatherCache::new(cache_ttl_seconds),
        }
    }

    pub async fn fetch_weather(&mut self, location: &str) -> Result<WeatherData, WeatherError> {
        if let Some(cached) = self.cache.get(location) {
            return Ok(cached);
        }

        if location.trim().is_empty() {
            return Err(WeatherError::InvalidLocation);
        }

        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
            location, self.api_key
        );

        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(WeatherError::Api(format!("HTTP {}: {}", status, text)));
        }

        let api_data: ApiResponse = response.json().await?;
        
        let weather_data = WeatherData {
            location: api_data.name,
            temperature: api_data.main.temp,
            humidity: api_data.main.humidity,
            pressure: api_data.main.pressure,
            description: api_data.weather.first()
                .map(|w| w.description.clone())
                .unwrap_or_default(),
            icon_code: api_data.weather.first()
                .map(|w| w.icon.clone())
                .unwrap_or_default(),
        };

        self.cache.insert(location.to_string(), weather_data.clone());
        self.cache.clear_expired();

        Ok(weather_data)
    }

    pub fn cache_stats(&self) -> (usize, Duration) {
        (self.cache.entries.len(), self.cache.ttl)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, server_url};

    #[tokio::test]
    async fn test_fetch_weather_success() {
        let _m = mock("GET", "/data/2.5/weather?q=London&appid=test_key&units=metric")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "name": "London",
                "main": {"temp": 15.5, "humidity": 65, "pressure": 1013},
                "weather": [{"description": "clear sky", "icon": "01d"}]
            }"#)
            .create();

        let mut fetcher = WeatherFetcher::new("test_key".to_string(), 300);
        fetcher.client = reqwest::Client::new();
        
        let result = fetcher.fetch_weather("London").await;
        assert!(result.is_ok());
        
        let data = result.unwrap();
        assert_eq!(data.location, "London");
        assert_eq!(data.temperature, 15.5);
        assert_eq!(data.description, "clear sky");
    }

    #[test]
    fn test_cache_operations() {
        let mut cache = WeatherCache::new(60);
        let test_data = WeatherData {
            location: "Test".to_string(),
            temperature: 20.0,
            humidity: 50,
            pressure: 1000,
            description: "test".to_string(),
            icon_code: "01d".to_string(),
        };

        assert!(cache.get("Test").is_none());
        
        cache.insert("Test".to_string(), test_data.clone());
        assert!(cache.get("Test").is_some());
        
        cache.clear_expired();
        assert!(cache.get("Test").is_some());
    }
}