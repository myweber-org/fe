
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

#[derive(Deserialize, Debug, Clone)]
struct WeatherData {
    main: MainData,
    weather: Vec<WeatherInfo>,
    name: String,
}

#[derive(Deserialize, Debug, Clone)]
struct MainData {
    temp: f64,
    humidity: u8,
    pressure: u16,
}

#[derive(Deserialize, Debug, Clone)]
struct WeatherInfo {
    main: String,
    description: String,
}

struct CachedWeather {
    data: WeatherData,
    timestamp: SystemTime,
}

pub struct WeatherFetcher {
    api_key: String,
    cache: HashMap<String, CachedWeather>,
    cache_duration: Duration,
    client: reqwest::Client,
}

impl WeatherFetcher {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            cache: HashMap::new(),
            cache_duration: Duration::from_secs(300),
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_weather(&mut self, city: &str) -> Result<WeatherData, WeatherError> {
        if let Some(cached) = self.cache.get(city) {
            if cached.timestamp.elapsed().unwrap_or(self.cache_duration) < self.cache_duration {
                return Ok(cached.data.clone());
            }
        }

        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
            city, self.api_key
        );

        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            return Err(WeatherError::InvalidResponse);
        }

        let weather_data: WeatherData = response.json().await?;
        
        self.cache.insert(
            city.to_string(),
            CachedWeather {
                data: weather_data.clone(),
                timestamp: SystemTime::now(),
            },
        );

        Ok(weather_data)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn set_cache_duration(&mut self, duration: Duration) {
        self.cache_duration = duration;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito;

    #[tokio::test]
    async fn test_weather_fetch() {
        let mut server = mockito::Server::new();
        let mock = server.mock("GET", "/data/2.5/weather")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"main":{"temp":20.5,"humidity":65,"pressure":1013},"weather":[{"main":"Clear","description":"clear sky"}],"name":"London"}"#)
            .create();

        let mut fetcher = WeatherFetcher::new("test_key".to_string());
        fetcher.client = reqwest::Client::builder()
            .base_url(server.url())
            .build()
            .unwrap();

        let result = fetcher.get_weather("London").await;
        assert!(result.is_ok());
        
        let weather = result.unwrap();
        assert_eq!(weather.name, "London");
        assert_eq!(weather.main.temp, 20.5);
        
        mock.assert();
    }
}