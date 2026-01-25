
use reqwest;
use serde::Deserialize;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct WeatherData {
    temperature: f64,
    humidity: f64,
    wind_speed: f64,
    conditions: String,
}

#[derive(Error, Debug)]
pub enum WeatherError {
    #[error("Network request failed: {0}")]
    Network(#[from] reqwest::Error),
    #[error("API returned error: {0}")]
    Api(String),
    #[error("Invalid response format")]
    ParseError,
}

pub struct WeatherFetcher {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
}

impl WeatherFetcher {
    pub fn new(api_key: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");
        
        WeatherFetcher {
            client,
            api_key,
            base_url: "https://api.weather.example.com".to_string(),
        }
    }

    pub async fn fetch_weather(&self, city: &str) -> Result<WeatherData, WeatherError> {
        let url = format!("{}/v1/current?city={}&key={}", self.base_url, city, self.api_key);
        
        for attempt in 1..=3 {
            match self.client.get(&url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        let weather: WeatherData = response.json().await
                            .map_err(|_| WeatherError::ParseError)?;
                        return Ok(weather);
                    } else {
                        if attempt == 3 {
                            return Err(WeatherError::Api(
                                format!("HTTP {}: {}", response.status(), response.text().await?)
                            ));
                        }
                    }
                }
                Err(e) => {
                    if attempt == 3 {
                        return Err(WeatherError::Network(e));
                    }
                }
            }
            
            tokio::time::sleep(Duration::from_secs(attempt as u64)).await;
        }
        
        unreachable!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, server_url};

    #[tokio::test]
    async fn test_successful_fetch() {
        let _m = mock("GET", "/v1/current?city=London&key=test_key")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"temperature":15.5,"humidity":65.0,"wind_speed":12.3,"conditions":"Partly Cloudy"}"#)
            .create();

        let mut fetcher = WeatherFetcher::new("test_key".to_string());
        fetcher.base_url = server_url();
        
        let result = fetcher.fetch_weather("London").await;
        assert!(result.is_ok());
        
        let weather = result.unwrap();
        assert_eq!(weather.temperature, 15.5);
        assert_eq!(weather.conditions, "Partly Cloudy");
    }

    #[tokio::test]
    async fn test_api_error() {
        let _m = mock("GET", "/v1/current?city=InvalidCity&key=test_key")
            .with_status(404)
            .with_body("City not found")
            .create();

        let mut fetcher = WeatherFetcher::new("test_key".to_string());
        fetcher.base_url = server_url();
        
        let result = fetcher.fetch_weather("InvalidCity").await;
        assert!(matches!(result, Err(WeatherError::Api(_))));
    }
}