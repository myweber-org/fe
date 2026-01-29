
use reqwest;
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
struct WeatherData {
    main: Main,
    name: String,
}

#[derive(Debug, Deserialize)]
struct Main {
    temp: f64,
    humidity: u8,
}

pub async fn get_weather(city: &str, api_key: &str) -> Result<WeatherData, Box<dyn Error>> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );

    let response = reqwest::get(&url).await?;
    let weather: WeatherData = response.json().await?;

    Ok(weather)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, server_url};

    #[tokio::test]
    async fn test_get_weather_success() {
        let mock_response = r#"{
            "main": {"temp": 22.5, "humidity": 65},
            "name": "London"
        }"#;

        let _m = mock("GET", "/data/2.5/weather?q=London&appid=test_key&units=metric")
            .with_status(200)
            .with_body(mock_response)
            .create();

        let api_key = "test_key";
        let result = get_weather("London", api_key).await;

        assert!(result.is_ok());
        let weather = result.unwrap();
        assert_eq!(weather.name, "London");
        assert_eq!(weather.main.temp, 22.5);
        assert_eq!(weather.main.humidity, 65);
    }
}