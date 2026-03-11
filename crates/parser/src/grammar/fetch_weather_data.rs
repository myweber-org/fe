
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WeatherError {
    #[error("Network request failed: {0}")]
    NetworkError(String),
    #[error("Invalid API response: {0}")]
    ParseError(String),
    #[error("Cache expired")]
    CacheExpired,
    #[error("Location not found")]
    LocationNotFound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherData {
    pub temperature: f64,
    pub humidity: f64,
    pub wind_speed: f64,
    pub description: String,
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone)]
pub struct WeatherCache {
    data: HashMap<String, (WeatherData, SystemTime)>,
    ttl: Duration,
}

impl WeatherCache {
    pub fn new(ttl_seconds: u64) -> Self {
        WeatherCache {
            data: HashMap::new(),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    pub fn get(&self, location: &str) -> Option<WeatherData> {
        self.data.get(location).and_then(|(data, timestamp)| {
            if timestamp.elapsed().unwrap_or(self.ttl) < self.ttl {
                Some(data.clone())
            } else {
                None
            }
        })
    }

    pub fn set(&mut self, location: String, data: WeatherData) {
        self.data.insert(location, (data, SystemTime::now()));
    }

    pub fn clear_expired(&mut self) {
        let now = SystemTime::now();
        self.data.retain(|_, (_, timestamp)| {
            now.duration_since(*timestamp).unwrap_or(self.ttl) < self.ttl
        });
    }
}

pub struct WeatherFetcher {
    api_key: String,
    base_url: String,
    cache: WeatherCache,
}

impl WeatherFetcher {
    pub fn new(api_key: String, cache_ttl_seconds: u64) -> Self {
        WeatherFetcher {
            api_key,
            base_url: "https://api.weather.example.com".to_string(),
            cache: WeatherCache::new(cache_ttl_seconds),
        }
    }

    pub async fn fetch_weather(&mut self, location: &str) -> Result<WeatherData, WeatherError> {
        if let Some(cached) = self.cache.get(location) {
            return Ok(cached);
        }

        let weather_data = self.fetch_from_api(location).await?;
        self.cache.set(location.to_string(), weather_data.clone());
        Ok(weather_data)
    }

    async fn fetch_from_api(&self, location: &str) -> Result<WeatherData, WeatherError> {
        let client = reqwest::Client::new();
        let url = format!(
            "{}/v1/weather?location={}&api_key={}",
            self.base_url, location, self.api_key
        );

        let response = client
            .get(&url)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| WeatherError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            if response.status() == 404 {
                return Err(WeatherError::LocationNotFound);
            }
            return Err(WeatherError::NetworkError(
                format!("HTTP {}: {}", response.status(), response.text().await.unwrap_or_default())
            ));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| WeatherError::ParseError(e.to_string()))?;

        Ok(WeatherData {
            temperature: json["main"]["temp"]
                .as_f64()
                .ok_or_else(|| WeatherError::ParseError("Invalid temperature".to_string()))?,
            humidity: json["main"]["humidity"]
                .as_f64()
                .ok_or_else(|| WeatherError::ParseError("Invalid humidity".to_string()))?,
            wind_speed: json["wind"]["speed"]
                .as_f64()
                .ok_or_else(|| WeatherError::ParseError("Invalid wind speed".to_string()))?,
            description: json["weather"][0]["description"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
            timestamp: SystemTime::now(),
        })
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear_expired();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, Matcher};

    #[tokio::test]
    async fn test_fetch_weather_success() {
        let _m = mock("GET", "/v1/weather")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("location".into(), "London".into()),
                Matcher::UrlEncoded("api_key".into(), "test_key".into()),
            ]))
            .with_status(200)
            .with_body(r#"{
                "main": {"temp": 15.5, "humidity": 65.0},
                "wind": {"speed": 5.2},
                "weather": [{"description": "clear sky"}]
            }"#)
            .create();

        let mut fetcher = WeatherFetcher::new("test_key".to_string(), 300);
        fetcher.base_url = mockito::server_url();
        
        let result = fetcher.fetch_weather("London").await;
        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.temperature, 15.5);
        assert_eq!(data.humidity, 65.0);
        assert_eq!(data.wind_speed, 5.2);
        assert_eq!(data.description, "clear sky");
    }

    #[test]
    fn test_cache_operations() {
        let mut cache = WeatherCache::new(60);
        let test_data = WeatherData {
            temperature: 20.0,
            humidity: 50.0,
            wind_speed: 3.0,
            description: "test".to_string(),
            timestamp: SystemTime::now(),
        };

        cache.set("Paris".to_string(), test_data.clone());
        
        let cached = cache.get("Paris");
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().temperature, 20.0);

        let missing = cache.get("Berlin");
        assert!(missing.is_none());
    }
}