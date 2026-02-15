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
}use reqwest;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct WeatherResponse {
    main: MainData,
    weather: Vec<WeatherInfo>,
    name: String,
}

#[derive(Debug, Deserialize)]
struct MainData {
    temp: f64,
    humidity: u8,
    pressure: u16,
}

#[derive(Debug, Deserialize)]
struct WeatherInfo {
    main: String,
    description: String,
}

pub struct WeatherFetcher {
    api_key: String,
    client: reqwest::Client,
}

impl WeatherFetcher {
    pub fn new(api_key: &str) -> Self {
        WeatherFetcher {
            api_key: api_key.to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_weather(&self, city: &str) -> Result<WeatherResponse, reqwest::Error> {
        let mut params = HashMap::new();
        params.insert("q", city);
        params.insert("appid", &self.api_key);
        params.insert("units", "metric");

        let response = self.client
            .get("https://api.openweathermap.org/data/2.5/weather")
            .query(&params)
            .send()
            .await?
            .json::<WeatherResponse>()
            .await?;

        Ok(response)
    }

    pub fn format_weather_data(weather: &WeatherResponse) -> String {
        format!(
            "Weather in {}: {:.1}°C, {} ({}), Humidity: {}%, Pressure: {} hPa",
            weather.name,
            weather.main.temp,
            weather.weather.first().map(|w| &w.main).unwrap_or(&"Unknown".to_string()),
            weather.weather.first().map(|w| &w.description).unwrap_or(&"".to_string()),
            weather.main.humidity,
            weather.main.pressure
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    #[tokio::test]
    async fn test_weather_fetch() {
        let mut server = Server::new_async().await;
        let mock = server.mock("GET", "/data/2.5/weather")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "name": "TestCity",
                "main": {"temp": 22.5, "humidity": 65, "pressure": 1013},
                "weather": [{"main": "Clear", "description": "clear sky"}]
            }"#)
            .create_async()
            .await;

        let fetcher = WeatherFetcher::new("test_key");
        let weather = fetcher.get_weather("TestCity").await.unwrap();
        
        assert_eq!(weather.name, "TestCity");
        assert_eq!(weather.main.temp, 22.5);
        assert_eq!(weather.weather[0].main, "Clear");
        
        mock.assert_async().await;
    }
}