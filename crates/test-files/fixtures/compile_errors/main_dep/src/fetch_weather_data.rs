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

async fn fetch_weather_data(api_key: &str, city: &str) -> Result<WeatherResponse, WeatherError> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(WeatherError::Network)?;

    let mut retries = 3;
    while retries > 0 {
        match client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let body = response.text().await.map_err(WeatherError::Network)?;
                    return serde_json::from_str(&body).map_err(WeatherError::Parse);
                } else if response.status().is_server_error() && retries > 1 {
                    retries -= 1;
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    continue;
                } else {
                    return Err(WeatherError::Network(response.error_for_status().unwrap_err()));
                }
            }
            Err(e) if retries > 1 => {
                retries -= 1;
                tokio::time::sleep(Duration::from_secs(2)).await;
                continue;
            }
            Err(e) => return Err(WeatherError::Network(e)),
        }
    }
    Err(WeatherError::RetryExhausted)
}

#[tokio::main]
async fn main() {
    let api_key = std::env::var("WEATHER_API_KEY").unwrap_or_else(|_| "demo_key".to_string());
    let city = "London";

    match fetch_weather_data(&api_key, city).await {
        Ok(weather) => {
            println!("Weather in {}: {:.1}°C, {}% humidity", 
                     weather.name, weather.main.temp, weather.main.humidity);
        }
        Err(e) => {
            eprintln!("Failed to fetch weather data: {:?}", e);
        }
    }
}