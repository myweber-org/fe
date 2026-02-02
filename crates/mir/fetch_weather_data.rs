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
    InvalidApiKey,
}

async fn fetch_weather_data(
    api_key: &str,
    city: &str,
    max_retries: u8,
) -> Result<WeatherResponse, WeatherError> {
    if api_key.is_empty() {
        return Err(WeatherError::InvalidApiKey);
    }

    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(WeatherError::Network)?;

    for attempt in 1..=max_retries {
        match client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let weather_data: WeatherResponse = response
                        .json()
                        .await
                        .map_err(WeatherError::Parse)?;
                    return Ok(weather_data);
                } else if response.status() == 401 {
                    return Err(WeatherError::InvalidApiKey);
                }
            }
            Err(e) if attempt == max_retries => {
                return Err(WeatherError::Network(e));
            }
            _ => {
                tokio::time::sleep(Duration::from_secs(2u64.pow(attempt as u32))).await;
                continue;
            }
        }
    }

    Err(WeatherError::Network(
        reqwest::Error::new(reqwest::StatusCode::REQUEST_TIMEOUT, "Max retries exceeded")
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, Matcher};

    #[tokio::test]
    async fn test_fetch_weather_success() {
        let _m = mock("GET", "/data/2.5/weather")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("q".into(), "London".into()),
                Matcher::UrlEncoded("appid".into(), "test_key".into()),
                Matcher::UrlEncoded("units".into(), "metric".into()),
            ]))
            .with_status(200)
            .with_body(r#"{"name":"London","main":{"temp":15.5,"humidity":65}}"#)
            .create();

        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        let result = fetch_weather_data("test_key", "London", 3).await;
        assert!(result.is_ok());
        
        let weather = result.unwrap();
        assert_eq!(weather.name, "London");
        assert_eq!(weather.main.temp, 15.5);
        assert_eq!(weather.main.humidity, 65);
    }

    #[tokio::test]
    async fn test_fetch_weather_invalid_api_key() {
        let result = fetch_weather_data("", "London", 3).await;
        assert!(matches!(result, Err(WeatherError::InvalidApiKey)));
    }
}