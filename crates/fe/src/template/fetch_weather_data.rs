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
    weather: Vec<WeatherInfo>,
    name: String,
}

#[derive(Deserialize, Debug)]
struct MainData {
    temp: f64,
    humidity: u8,
    pressure: u16,
}

#[derive(Deserialize, Debug)]
struct WeatherInfo {
    description: String,
    icon: String,
}

pub struct WeatherClient {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
}

impl WeatherClient {
    pub fn new(api_key: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");
        
        WeatherClient {
            client,
            api_key,
            base_url: "https://api.openweathermap.org/data/2.5".to_string(),
        }
    }

    pub async fn get_current_weather(&self, city: &str) -> Result<WeatherData, WeatherError> {
        let url = format!(
            "{}/weather?q={}&appid={}&units=metric",
            self.base_url, city, self.api_key
        );

        let response = self.client
            .get(&url)
            .send()
            .await?
            .error_for_status()
            .map_err(|e| WeatherError::ApiError(e.to_string()))?;

        let weather_response: WeatherResponse = response
            .json()
            .await
            .map_err(|_| WeatherError::InvalidFormat)?;

        Ok(WeatherData {
            city: weather_response.name,
            temperature: weather_response.main.temp,
            humidity: weather_response.main.humidity,
            pressure: weather_response.main.pressure,
            description: weather_response.weather
                .first()
                .map(|w| w.description.clone())
                .unwrap_or_default(),
            icon_code: weather_response.weather
                .first()
                .map(|w| w.icon.clone())
                .unwrap_or_default(),
        })
    }

    pub async fn get_weather_with_retry(
        &self,
        city: &str,
        max_retries: u8
    ) -> Result<WeatherData, WeatherError> {
        for attempt in 1..=max_retries {
            match self.get_current_weather(city).await {
                Ok(data) => return Ok(data),
                Err(e) if attempt == max_retries => return Err(e),
                Err(_) => {
                    tokio::time::sleep(Duration::from_secs(attempt as u64)).await;
                    continue;
                }
            }
        }
        Err(WeatherError::ApiError("Max retries exceeded".to_string()))
    }
}

#[derive(Debug, Clone)]
pub struct WeatherData {
    pub city: String,
    pub temperature: f64,
    pub humidity: u8,
    pub pressure: u16,
    pub description: String,
    pub icon_code: String,
}

impl WeatherData {
    pub fn format_summary(&self) -> String {
        format!(
            "Weather in {}: {:.1}°C, {}, Humidity: {}%, Pressure: {} hPa",
            self.city, self.temperature, self.description, self.humidity, self.pressure
        )
    }

    pub fn is_cold(&self) -> bool {
        self.temperature < 10.0
    }

    pub fn is_hot(&self) -> bool {
        self.temperature > 30.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, Matcher};

    #[tokio::test]
    async fn test_weather_data_formatting() {
        let data = WeatherData {
            city: "London".to_string(),
            temperature: 15.5,
            humidity: 65,
            pressure: 1013,
            description: "clear sky".to_string(),
            icon_code: "01d".to_string(),
        };

        assert_eq!(
            data.format_summary(),
            "Weather in London: 15.5°C, clear sky, Humidity: 65%, Pressure: 1013 hPa"
        );
        assert!(!data.is_cold());
        assert!(!data.is_hot());
    }

    #[tokio::test]
    async fn test_mock_weather_request() {
        let _m = mock("GET", "/weather")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("q".into(), "London".into()),
                Matcher::UrlEncoded("appid".into(), "test_key".into()),
                Matcher::UrlEncoded("units".into(), "metric".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "name": "London",
                "main": {"temp": 15.5, "humidity": 65, "pressure": 1013},
                "weather": [{"description": "clear sky", "icon": "01d"}]
            }"#)
            .create();

        let client = WeatherClient::new("test_key".to_string());
        let result = client.get_current_weather("London").await;
        
        assert!(result.is_ok());
        let weather = result.unwrap();
        assert_eq!(weather.city, "London");
        assert_eq!(weather.temperature, 15.5);
    }
}