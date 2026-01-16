use std::error::Error;

const MOCK_API_URL: &str = "https://api.mockweather.example.com/v1/current";

#[derive(Debug, serde::Deserialize)]
struct WeatherData {
    city: String,
    temperature_c: f64,
    condition: String,
    humidity: u8,
}

async fn fetch_weather(city: &str) -> Result<WeatherData, Box<dyn Error>> {
    let url = format!("{}?city={}", MOCK_API_URL, city);
    let response = reqwest::get(&url).await?;
    
    if !response.status().is_success() {
        return Err(format!("API request failed with status: {}", response.status()).into());
    }
    
    let weather: WeatherData = response.json().await?;
    Ok(weather)
}

fn display_weather(data: &WeatherData) {
    println!("Weather in {}:", data.city);
    println!("  Temperature: {:.1}°C", data.temperature_c);
    println!("  Condition: {}", data.condition);
    println!("  Humidity: {}%", data.humidity);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <city>", args[0]);
        std::process::exit(1);
    }
    
    let city = &args[1];
    match fetch_weather(city).await {
        Ok(weather) => display_weather(&weather),
        Err(e) => eprintln!("Error fetching weather data: {}", e),
    }
    
    Ok(())
}