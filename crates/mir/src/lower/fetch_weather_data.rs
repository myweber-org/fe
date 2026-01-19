use reqwest;
use serde::Deserialize;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct WeatherResponse {
    main: MainData,
    name: String,
}

#[derive(Debug, Deserialize)]
struct MainData {
    temp: f64,
    humidity: u8,
}

#[derive(Error, Debug)]
pub enum WeatherError {
    #[error("Network request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("API returned error: {0}")]
    ApiError(String),
    #[error("Maximum retry attempts exceeded")]
    MaxRetriesExceeded,
}

pub async fn fetch_weather_data(api_key: &str, city: &str) -> Result<WeatherResponse, WeatherError> {
    const MAX_RETRIES: u8 = 3;
    const RETRY_DELAY: Duration = Duration::from_secs(1);
    
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );
    
    let mut last_error = None;
    
    for attempt in 0..MAX_RETRIES {
        match reqwest::get(&url).await {
            Ok(response) => {
                if response.status().is_success() {
                    return response.json::<WeatherResponse>().await.map_err(WeatherError::from);
                } else {
                    let status = response.status();
                    let error_text = response.text().await.unwrap_or_default();
                    last_error = Some(format!("Status: {}, Body: {}", status, error_text));
                }
            }
            Err(e) => {
                last_error = Some(e.to_string());
            }
        }
        
        if attempt < MAX_RETRIES - 1 {
            tokio::time::sleep(RETRY_DELAY).await;
        }
    }
    
    Err(WeatherError::ApiError(
        last_error.unwrap_or_else(|| "Unknown error".to_string())
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, Matcher};

    #[tokio::test]
    async fn test_successful_fetch() {
        let _m = mock("GET", "/data/2.5/weather")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("q".into(), "London".into()),
                Matcher::UrlEncoded("appid".into(), "test_key".into()),
                Matcher::UrlEncoded("units".into(), "metric".into()),
            ]))
            .with_status(200)
            .with_body(r#"{"name":"London","main":{"temp":15.5,"humidity":65}}"#)
            .create();
        
        let result = fetch_weather_data("test_key", "London").await;
        assert!(result.is_ok());
        let weather = result.unwrap();
        assert_eq!(weather.name, "London");
        assert_eq!(weather.main.temp, 15.5);
        assert_eq!(weather.main.humidity, 65);
    }
}