use reqwest;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WeatherError {
    #[error("Network request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("Invalid API response")]
    InvalidResponse,
    #[error("Cache expired")]
    CacheExpired,
}

#[derive(Debug, Deserialize, Clone)]
struct WeatherData {
    main: MainData,
    weather: Vec<WeatherInfo>,
    name: String,
}

#[derive(Debug, Deserialize, Clone)]
struct MainData {
    temp: f64,
    humidity: u8,
    pressure: u16,
}

#[derive(Debug, Deserialize, Clone)]
struct WeatherInfo {
    description: String,
    icon: String,
}

struct WeatherCache {
    data: HashMap<String, (WeatherData, SystemTime)>,
    ttl: Duration,
}

impl WeatherCache {
    fn new(ttl_seconds: u64) -> Self {
        Self {
            data: HashMap::new(),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    fn get(&self, city: &str) -> Option<WeatherData> {
        self.data.get(city).and_then(|(data, timestamp)| {
            if timestamp.elapsed().ok()? < self.ttl {
                Some(data.clone())
            } else {
                None
            }
        })
    }

    fn insert(&mut self, city: String, data: WeatherData) {
        self.data.insert(city, (data, SystemTime::now()));
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

    pub async fn fetch_weather(&mut self, city: &str) -> Result<WeatherData, WeatherError> {
        if let Some(cached) = self.cache.get(city) {
            return Ok(cached);
        }

        let url = format!(
            "{}?q={}&appid={}&units=metric",
            self.base_url, city, self.api_key
        );

        let response = reqwest::get(&url).await?;
        
        if !response.status().is_success() {
            return Err(WeatherError::InvalidResponse);
        }

        let weather_data: WeatherData = response.json().await?;
        self.cache.insert(city.to_string(), weather_data.clone());

        Ok(weather_data)
    }

    pub fn format_weather_report(data: &WeatherData) -> String {
        format!(
            "Weather in {}: {:.1}°C, {}% humidity, {} hPa. Conditions: {}",
            data.name,
            data.main.temp,
            data.main.humidity,
            data.main.pressure,
            data.weather.first()
                .map(|w| w.description.as_str())
                .unwrap_or("unknown")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    #[tokio::test]
    async fn test_fetch_weather_success() {
        let mut server = Server::new_async().await;
        let mock = server.mock("GET", "/data/2.5/weather")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "name": "London",
                "main": {"temp": 15.5, "humidity": 65, "pressure": 1013},
                "weather": [{"description": "clear sky", "icon": "01d"}]
            }"#)
            .create_async()
            .await;

        let mut fetcher = WeatherFetcher {
            api_key: "test_key".to_string(),
            base_url: server.url(),
            cache: WeatherCache::new(300),
        };

        let result = fetcher.fetch_weather("London").await;
        assert!(result.is_ok());
        
        let data = result.unwrap();
        assert_eq!(data.name, "London");
        assert_eq!(data.main.temp, 15.5);
        
        mock.assert_async().await;
    }

    #[test]
    fn test_format_weather_report() {
        let data = WeatherData {
            name: "Paris".to_string(),
            main: MainData {
                temp: 22.3,
                humidity: 70,
                pressure: 1015,
            },
            weather: vec![WeatherInfo {
                description: "partly cloudy".to_string(),
                icon: "02d".to_string(),
            }],
        };

        let report = WeatherFetcher::format_weather_report(&data);
        assert!(report.contains("Weather in Paris"));
        assert!(report.contains("22.3°C"));
        assert!(report.contains("partly cloudy"));
    }
}