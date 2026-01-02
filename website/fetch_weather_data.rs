use reqwest;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct WeatherResponse {
    main: Main,
    name: String,
}

#[derive(Debug, Deserialize)]
struct Main {
    temp: f64,
    humidity: u8,
}

#[derive(Debug)]
enum WeatherError {
    Network(reqwest::Error),
    Parse(serde_json::Error),
    RetryExhausted,
}

async fn fetch_weather_data(api_key: &str, city: &str, max_retries: u8) -> Result<WeatherResponse, WeatherError> {
    let client = reqwest::Client::new();
    let url = format!("https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric", city, api_key);
    
    for attempt in 1..=max_retries {
        match client.get(&url).timeout(Duration::from_secs(10)).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let body = response.text().await.map_err(WeatherError::Network)?;
                    return serde_json::from_str(&body).map_err(WeatherError::Parse);
                } else if attempt < max_retries {
                    tokio::time::sleep(Duration::from_secs(1 << attempt)).await;
                    continue;
                }
            }
            Err(e) if attempt < max_retries => {
                tokio::time::sleep(Duration::from_secs(1 << attempt)).await;
                continue;
            }
            Err(e) => return Err(WeatherError::Network(e)),
        }
    }
    
    Err(WeatherError::RetryExhausted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, server_url};

    #[tokio::test]
    async fn test_fetch_weather_success() {
        let _m = mock("GET", "/data/2.5/weather")
            .with_status(200)
            .with_body(r#"{"name":"London","main":{"temp":15.5,"humidity":65}}"#)
            .create();
        
        let result = fetch_weather_data("test_key", "London", 3).await;
        assert!(result.is_ok());
    }
}