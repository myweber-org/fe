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

    let mut attempts = 0;
    let max_attempts = 3;

    while attempts < max_attempts {
        match client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let text = response.text().await.map_err(WeatherError::Network)?;
                    return serde_json::from_str(&text).map_err(WeatherError::Parse);
                } else if response.status().is_server_error() && attempts < max_attempts - 1 {
                    attempts += 1;
                    tokio::time::sleep(Duration::from_secs(1 << attempts)).await;
                    continue;
                } else {
                    return Err(WeatherError::Network(
                        response.error_for_status().unwrap_err(),
                    ));
                }
            }
            Err(e) if attempts < max_attempts - 1 => {
                attempts += 1;
                tokio::time::sleep(Duration::from_secs(1 << attempts)).await;
                continue;
            }
            Err(e) => return Err(WeatherError::Network(e)),
        }
    }

    Err(WeatherError::RetryExhausted)
}

#[tokio::main]
async fn main() {
    let api_key = "your_api_key_here";
    let city = "London";

    match fetch_weather_data(api_key, city).await {
        Ok(weather) => {
            println!("Weather in {}:", weather.name);
            println!("Temperature: {:.1}°C", weather.main.temp);
            println!("Humidity: {}%", weather.main.humidity);
        }
        Err(e) => eprintln!("Failed to fetch weather data: {:?}", e),
    }
}