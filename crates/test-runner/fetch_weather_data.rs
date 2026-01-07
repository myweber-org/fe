use reqwest;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct WeatherData {
    pub temperature: f64,
    pub humidity: u8,
    pub pressure: u16,
    pub description: String,
    pub city: String,
}

pub struct WeatherFetcher {
    api_key: String,
    client: reqwest::Client,
}

impl WeatherFetcher {
    pub fn new() -> Result<Self, String> {
        let api_key = env::var("OPENWEATHER_API_KEY")
            .map_err(|_| "OPENWEATHER_API_KEY environment variable not set".to_string())?;
        
        Ok(WeatherFetcher {
            api_key,
            client: reqwest::Client::new(),
        })
    }

    pub async fn fetch_current_weather(&self, city: &str) -> Result<WeatherData, String> {
        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
            city, self.api_key
        );

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("API request failed with status: {}", response.status()));
        }

        let json_response: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse JSON response: {}", e))?;

        let main_data = &json_response["main"];
        let weather_array = &json_response["weather"]
            .as_array()
            .ok_or("Weather data not found in response")?;

        let weather_description = weather_array
            .first()
            .and_then(|w| w["description"].as_str())
            .unwrap_or("Unknown")
            .to_string();

        Ok(WeatherData {
            temperature: main_data["temp"].as_f64().unwrap_or(0.0),
            humidity: main_data["humidity"].as_u64().unwrap_or(0) as u8,
            pressure: main_data["pressure"].as_u64().unwrap_or(0) as u16,
            description: weather_description,
            city: city.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito;

    #[tokio::test]
    async fn test_fetch_current_weather_success() {
        let mut server = mockito::Server::new_async().await;
        let mock = server.mock("GET", "/data/2.5/weather")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("q".into(), "London".into()),
                mockito::Matcher::UrlEncoded("appid".into(), "test_key".into()),
                mockito::Matcher::UrlEncoded("units".into(), "metric".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "main": {
                    "temp": 15.5,
                    "humidity": 65,
                    "pressure": 1013
                },
                "weather": [{
                    "description": "clear sky"
                }]
            }"#)
            .create_async()
            .await;

        env::set_var("OPENWEATHER_API_KEY", "test_key");
        let fetcher = WeatherFetcher::new().unwrap();
        
        let weather = fetcher.fetch_current_weather("London").await.unwrap();
        
        assert_eq!(weather.temperature, 15.5);
        assert_eq!(weather.humidity, 65);
        assert_eq!(weather.pressure, 1013);
        assert_eq!(weather.description, "clear sky");
        assert_eq!(weather.city, "London");
        
        mock.assert_async().await;
    }
}