use reqwest;
use serde::Deserialize;
use std::error::Error;

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

async fn fetch_weather_data(api_key: &str, city: &str) -> Result<WeatherResponse, Box<dyn Error>> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );
    
    let response = reqwest::get(&url).await?;
    
    if !response.status().is_success() {
        return Err(format!("API request failed with status: {}", response.status()).into());
    }
    
    let weather_data: WeatherResponse = response.json().await?;
    Ok(weather_data)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let api_key = "your_api_key_here";
    let city = "London";
    
    match fetch_weather_data(api_key, city).await {
        Ok(data) => {
            println!("Weather in {}:", data.name);
            println!("Temperature: {:.1}°C", data.main.temp);
            println!("Humidity: {}%", data.main.humidity);
        }
        Err(e) => eprintln!("Failed to fetch weather data: {}", e),
    }
    
    Ok(())
}
use reqwest;
use serde::Deserialize;
use std::error::Error;

#[derive(Deserialize, Debug)]
struct WeatherData {
    name: String,
    main: Main,
    weather: Vec<Weather>,
}

#[derive(Deserialize, Debug)]
struct Main {
    temp: f64,
    humidity: u8,
}

#[derive(Deserialize, Debug)]
struct Weather {
    description: String,
}

pub async fn get_weather(city: &str, api_key: &str) -> Result<WeatherData, Box<dyn Error>> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );
    
    let response = reqwest::get(&url).await?;
    
    if !response.status().is_success() {
        return Err(format!("API request failed with status: {}", response.status()).into());
    }
    
    let weather_data: WeatherData = response.json().await?;
    Ok(weather_data)
}

pub fn display_weather(data: &WeatherData) {
    println!("Weather in {}:", data.name);
    println!("Temperature: {:.1}°C", data.main.temp);
    println!("Humidity: {}%", data.main.humidity);
    
    if let Some(weather) = data.weather.first() {
        println!("Conditions: {}", weather.description);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, server_url};

    #[tokio::test]
    async fn test_get_weather_success() {
        let mock = mock("GET", "/data/2.5/weather?q=London&appid=test_key&units=metric")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "name": "London",
                "main": {"temp": 15.5, "humidity": 65},
                "weather": [{"description": "clear sky"}]
            }"#)
            .create();

        let _guard = mockito::server_guard();
        let url = server_url();
        
        let result = get_weather("London", "test_key").await;
        assert!(result.is_ok());
        
        let data = result.unwrap();
        assert_eq!(data.name, "London");
        assert_eq!(data.main.temp, 15.5);
        assert_eq!(data.main.humidity, 65);
        assert_eq!(data.weather[0].description, "clear sky");
    }
}use reqwest;
use serde::Deserialize;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct WeatherData {
    temperature: f64,
    humidity: f64,
    wind_speed: f64,
    description: String,
}

#[derive(Error, Debug)]
pub enum WeatherError {
    #[error("Network request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("API returned error: {0}")]
    ApiError(String),
    #[error("Invalid response format")]
    InvalidFormat,
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
        
        Self {
            client,
            api_key,
            base_url: "https://api.weather.example.com".to_string(),
        }
    }

    pub async fn fetch_weather(&self, city: &str, retries: u8) -> Result<WeatherData, WeatherError> {
        let mut last_error = None;
        
        for attempt in 0..=retries {
            match self.try_fetch(city).await {
                Ok(data) => return Ok(data),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < retries {
                        tokio::time::sleep(Duration::from_millis(200 * 2u64.pow(attempt as u32))).await;
                        continue;
                    }
                }
            }
        }
        
        Err(last_error.unwrap())
    }

    async fn try_fetch(&self, city: &str) -> Result<WeatherData, WeatherError> {
        let url = format!("{}/v1/weather?city={}&api_key={}", self.base_url, city, self.api_key);
        
        let response = self.client
            .get(&url)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(WeatherError::ApiError(error_text));
        }
        
        let json: serde_json::Value = response.json().await?;
        
        Ok(WeatherData {
            temperature: json["main"]["temp"].as_f64().ok_or(WeatherError::InvalidFormat)?,
            humidity: json["main"]["humidity"].as_f64().ok_or(WeatherError::InvalidFormat)?,
            wind_speed: json["wind"]["speed"].as_f64().ok_or(WeatherError::InvalidFormat)?,
            description: json["weather"][0]["description"]
                .as_str()
                .ok_or(WeatherError::InvalidFormat)?
                .to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, Matcher};

    #[tokio::test]
    async fn test_fetch_weather_success() {
        let _m = mock("GET", "/v1/weather")
            .match_query(Matcher::Regex(r"city=London&api_key=.+".into()))
            .with_status(200)
            .with_body(r#"{
                "main": {"temp": 15.5, "humidity": 65.0},
                "wind": {"speed": 5.2},
                "weather": [{"description": "clear sky"}]
            }"#)
            .create();
        
        let client = WeatherClient::new("test_key".to_string());
        let result = client.fetch_weather("London", 3).await;
        
        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.temperature, 15.5);
        assert_eq!(data.description, "clear sky");
    }
}
use reqwest;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct WeatherData {
    main: Main,
    name: String,
}

#[derive(Deserialize, Debug)]
struct Main {
    temp: f64,
    humidity: u8,
}

async fn fetch_weather(api_key: &str, city: &str) -> Result<WeatherData, reqwest::Error> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );
    
    let response = reqwest::get(&url).await?;
    let weather: WeatherData = response.json().await?;
    
    Ok(weather)
}

#[tokio::main]
async fn main() {
    let api_key = "your_api_key_here";
    let city = "London";
    
    match fetch_weather(api_key, city).await {
        Ok(data) => {
            println!("Weather in {}:", data.name);
            println!("Temperature: {:.1}°C", data.main.temp);
            println!("Humidity: {}%", data.main.humidity);
        }
        Err(e) => eprintln!("Failed to fetch weather data: {}", e),
    }
}