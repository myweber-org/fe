
use reqwest;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
pub struct WeatherData {
    temperature: f64,
    humidity: f64,
    description: String,
}

#[derive(Debug)]
pub enum WeatherError {
    NetworkError(reqwest::Error),
    ParseError(serde_json::Error),
    ApiError(String),
}

pub async fn fetch_weather_data(
    api_key: &str,
    city: &str,
    max_retries: u32,
) -> Result<WeatherData, WeatherError> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(WeatherError::NetworkError)?;

    let url = format!(
        "https://api.weather.example.com/data?city={}&apikey={}",
        city, api_key
    );

    for attempt in 0..max_retries {
        match client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let json_text = response.text().await.map_err(WeatherError::NetworkError)?;
                    return serde_json::from_str(&json_text).map_err(WeatherError::ParseError);
                } else if attempt < max_retries - 1 {
                    tokio::time::sleep(Duration::from_millis(500 * (attempt + 1) as u64)).await;
                    continue;
                } else {
                    return Err(WeatherError::ApiError(format!(
                        "HTTP {}: {}",
                        response.status(),
                        response.text().await.unwrap_or_default()
                    )));
                }
            }
            Err(e) => {
                if attempt < max_retries - 1 {
                    tokio::time::sleep(Duration::from_millis(500 * (attempt + 1) as u64)).await;
                    continue;
                }
                return Err(WeatherError::NetworkError(e));
            }
        }
    }

    Err(WeatherError::ApiError("Max retries exceeded".to_string()))
}