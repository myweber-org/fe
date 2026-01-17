use std::collections::HashMap;
use std::time::{Duration, Instant};
use reqwest::Error;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct WeatherData {
    temperature: f64,
    humidity: u8,
    conditions: String,
}

struct WeatherCache {
    data: HashMap<String, (WeatherData, Instant)>,
    ttl: Duration,
}

impl WeatherCache {
    fn new(ttl_seconds: u64) -> Self {
        WeatherCache {
            data: HashMap::new(),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    fn get(&self, city: &str) -> Option<&WeatherData> {
        self.data.get(city).and_then(|(data, timestamp)| {
            if timestamp.elapsed() < self.ttl {
                Some(data)
            } else {
                None
            }
        })
    }

    fn insert(&mut self, city: String, weather: WeatherData) {
        self.data.insert(city, (weather, Instant::now()));
    }
}

async fn fetch_weather(api_key: &str, city: &str) -> Result<WeatherData, Error> {
    let url = format!("https://api.weather.example.com/data?city={}&key={}", city, api_key);
    let response = reqwest::get(&url).await?;
    let weather: WeatherData = response.json().await?;
    Ok(weather)
}

pub async fn get_weather(cache: &mut WeatherCache, api_key: &str, city: &str) -> Result<WeatherData, String> {
    if let Some(cached) = cache.get(city) {
        return Ok(cached.clone());
    }

    match fetch_weather(api_key, city).await {
        Ok(weather) => {
            cache.insert(city.to_string(), weather.clone());
            Ok(weather)
        }
        Err(e) => Err(format!("Failed to fetch weather: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::mock;

    #[tokio::test]
    async fn test_weather_fetch() {
        let _m = mock("GET", "/data?city=London&key=test_key")
            .with_status(200)
            .with_body(r#"{"temperature":15.5,"humidity":65,"conditions":"Cloudy"}"#)
            .create();

        let mut cache = WeatherCache::new(300);
        let result = get_weather(&mut cache, "test_key", "London").await;
        assert!(result.is_ok());
    }
}