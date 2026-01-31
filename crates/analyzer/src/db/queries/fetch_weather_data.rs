
use reqwest;
use serde::Deserialize;

#[derive(Deserialize)]
struct WeatherData {
    main: Main,
    name: String,
}

#[derive(Deserialize)]
struct Main {
    temp: f64,
    humidity: u8,
}

pub async fn get_weather(api_key: &str, city: &str) -> Result<WeatherData, reqwest::Error> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );
    
    let response = reqwest::get(&url).await?;
    let weather: WeatherData = response.json().await?;
    
    Ok(weather)
}