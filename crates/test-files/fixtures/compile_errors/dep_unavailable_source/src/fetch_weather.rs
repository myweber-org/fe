use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherData {
    temperature: f64,
    humidity: f64,
    description: String,
    timestamp: SystemTime,
}

#[derive(Error, Debug)]
pub enum WeatherError {
    #[error("Network request failed: {0}")]
    NetworkError(String),
    #[error("Invalid API response: {0}")]
    ParseError(String),
    #[error("Cache expired")]
    CacheExpired,
}

pub struct WeatherFetcher {
    api_key: String,
    cache: HashMap<String, (WeatherData, SystemTime)>,
    cache_duration: Duration,
}

impl WeatherFetcher {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            cache: HashMap::new(),
            cache_duration: Duration::from_secs(300),
        }
    }

    pub fn set_cache_duration(&mut self, duration: Duration) {
        self.cache_duration = duration;
    }

    pub async fn fetch_weather(&mut self, city: &str) -> Result<WeatherData, WeatherError> {
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
            "https://api.weather.example.com/data?city={}&key={}",
            city, self.api_key
        );

        let response = reqwest::get(&url)
            .await
            .map_err(|e| WeatherError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(WeatherError::NetworkError(format!(
                "HTTP {}",
                response.status()
            )));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| WeatherError::ParseError(e.to_string()))?;

        let temperature = json["main"]["temp"]
            .as_f64()
            .ok_or_else(|| WeatherError::ParseError("Missing temperature".to_string()))?;

        let humidity = json["main"]["humidity"]
            .as_f64()
            .ok_or_else(|| WeatherError::ParseError("Missing humidity".to_string()))?;

        let description = json["weather"][0]["description"]
            .as_str()
            .ok_or_else(|| WeatherError::ParseError("Missing description".to_string()))?
            .to_string();

        Ok(WeatherData {
            temperature,
            humidity,
            description,
            timestamp: SystemTime::now(),
        })
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn remove_city(&mut self, city: &str) {
        self.cache.remove(city);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, Matcher};

    #[tokio::test]
    async fn test_fetch_weather_success() {
        let _m = mock("GET", "/data")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("city".into(), "London".into()),
                Matcher::UrlEncoded("key".into(), "test_key".into()),
            ]))
            .with_status(200)
            .with_body(r#"{"main":{"temp":15.5,"humidity":65},"weather":[{"description":"clear sky"}]}"#)
            .create();

        let mut fetcher = WeatherFetcher::new("test_key".to_string());
        fetcher.set_cache_duration(Duration::from_secs(1));

        let result = fetcher.fetch_weather("London").await;
        assert!(result.is_ok());
        
        let weather = result.unwrap();
        assert_eq!(weather.temperature, 15.5);
        assert_eq!(weather.humidity, 65.0);
        assert_eq!(weather.description, "clear sky");
    }

    #[tokio::test]
    async fn test_cache_behavior() {
        let mut fetcher = WeatherFetcher::new("test_key".to_string());
        fetcher.set_cache_duration(Duration::from_secs(3600));

        let city = "Berlin";
        
        let first_fetch = fetcher.fetch_weather(city).await;
        assert!(first_fetch.is_err());

        fetcher.clear_cache();
        assert!(fetcher.cache.is_empty());
    }
}