use reqwest;
use serde::Deserialize;
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WeatherError {
    #[error("Network request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("API returned error: {0}")]
    ApiError(String),
    #[error("Invalid response format")]
    InvalidFormat,
}

#[derive(Deserialize, Debug)]
struct WeatherResponse {
    main: MainData,
    name: String,
}

#[derive(Deserialize, Debug)]
struct MainData {
    temp: f64,
    humidity: u8,
    pressure: u16,
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
        
        Self {
            client,
            api_key,
            base_url: "https://api.openweathermap.org/data/2.5/weather".to_string(),
        }
    }

    pub async fn fetch_weather(&self, city: &str) -> Result<WeatherData, WeatherError> {
        let url = format!("{}?q={}&appid={}&units=metric", self.base_url, city, self.api_key);
        
        for attempt in 1..=3 {
            match self.client.get(&url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        let weather_response: WeatherResponse = response.json().await
                            .map_err(|_| WeatherError::InvalidFormat)?;
                        
                        return Ok(WeatherData {
                            city: weather_response.name,
                            temperature: weather_response.main.temp,
                            humidity: weather_response.main.humidity,
                            pressure: weather_response.main.pressure,
                        });
                    } else {
                        if attempt == 3 {
                            return Err(WeatherError::ApiError(
                                format!("HTTP {}: {}", response.status(), response.text().await.unwrap_or_default())
                            ));
                        }
                        tokio::time::sleep(Duration::from_secs(attempt as u64)).await;
                    }
                }
                Err(e) => {
                    if attempt == 3 {
                        return Err(WeatherError::RequestFailed(e));
                    }
                    tokio::time::sleep(Duration::from_secs(attempt as u64)).await;
                }
            }
        }
        
        unreachable!()
    }
}

#[derive(Debug, Clone)]
pub struct WeatherData {
    pub city: String,
    pub temperature: f64,
    pub humidity: u8,
    pub pressure: u16,
}

impl WeatherData {
    pub fn display(&self) -> String {
        format!(
            "Weather in {}: {:.1}°C, {}% humidity, {} hPa",
            self.city, self.temperature, self.humidity, self.pressure
        )
    }
}