use reqwest;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WeatherError {
    #[error("Network request failed: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("API response error: {0}")]
    ApiError(String),
    #[error("Invalid location provided")]
    InvalidLocation,
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

struct WeatherCache {
    data: HashMap<String, (WeatherResponse, SystemTime)>,
    ttl: Duration,
}

impl WeatherCache {
    fn new(ttl_seconds: u64) -> Self {
        Self {
            data: HashMap::new(),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    fn get(&self, location: &str) -> Option<&WeatherResponse> {
        self.data.get(location).and_then(|(response, timestamp)| {
            if timestamp.elapsed().unwrap_or(self.ttl) < self.ttl {
                Some(response)
            } else {
                None
            }
        })
    }

    fn insert(&mut self, location: String, response: WeatherResponse) {
        self.data.insert(location, (response, SystemTime::now()));
    }
}

pub struct WeatherFetcher {
    api_key: String,
    base_url: String,
    cache: WeatherCache,
}

impl WeatherFetcher {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.openweathermap.org/data/2.5/weather".to_string(),
            cache: WeatherCache::new(300),
        }
    }

    pub async fn fetch_weather(&mut self, location: &str) -> Result<WeatherResponse, WeatherError> {
        if location.trim().is_empty() {
            return Err(WeatherError::InvalidLocation);
        }

        if let Some(cached) = self.cache.get(location) {
            return Ok(cached.clone());
        }

        let url = format!(
            "{}?q={}&appid={}&units=metric",
            self.base_url, location, self.api_key
        );

        let response = reqwest::get(&url).await?;
        
        if !response.status().is_success() {
            return Err(WeatherError::ApiError(format!(
                "HTTP {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        let weather_data: WeatherResponse = response.json().await?;
        self.cache.insert(location.to_string(), weather_data.clone());
        
        Ok(weather_data)
    }

    pub fn display_weather(weather: &WeatherResponse) -> String {
        format!(
            "Weather in {}: {:.1}°C, {}% humidity, {} hPa. Conditions: {}",
            weather.name,
            weather.main.temp,
            weather.main.humidity,
            weather.main.pressure,
            weather.weather.first().map(|w| w.description.as_str()).unwrap_or("unknown")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_operations() {
        let mut cache = WeatherCache::new(60);
        let test_response = WeatherResponse {
            name: "TestCity".to_string(),
            main: MainData {
                temp: 20.0,
                humidity: 50,
                pressure: 1013,
            },
            weather: vec![WeatherInfo {
                description: "clear sky".to_string(),
                icon: "01d".to_string(),
            }],
        };

        cache.insert("TestCity".to_string(), test_response);
        assert!(cache.get("TestCity").is_some());
        assert!(cache.get("NonExistentCity").is_none());
    }

    #[test]
    fn test_invalid_location() {
        let fetcher = WeatherFetcher::new("test_key".to_string());
        let result = futures::executor::block_on(fetcher.fetch_weather(""));
        assert!(matches!(result, Err(WeatherError::InvalidLocation)));
    }
}