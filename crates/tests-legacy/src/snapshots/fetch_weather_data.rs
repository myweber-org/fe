
use reqwest;
use serde_json::Value;

pub async fn fetch_weather(city: &str, api_key: &str) -> Result<f64, Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );
    
    let response = reqwest::get(&url).await?;
    
    if !response.status().is_success() {
        return Err(format!("API request failed with status: {}", response.status()).into());
    }
    
    let json: Value = response.json().await?;
    
    match json["main"]["temp"].as_f64() {
        Some(temp) => Ok(temp),
        None => Err("Temperature data not found in response".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, server_url};

    #[tokio::test]
    async fn test_fetch_weather_success() {
        let _m = mock("GET", "/data/2.5/weather?q=London&appid=test_key&units=metric")
            .with_status(200)
            .with_body(r#"{"main":{"temp":15.5}}"#)
            .create();

        let temp = fetch_weather("London", "test_key").await;
        assert!(temp.is_ok());
        assert_eq!(temp.unwrap(), 15.5);
    }

    #[tokio::test]
    async fn test_fetch_weather_failure() {
        let _m = mock("GET", "/data/2.5/weather?q=InvalidCity&appid=test_key&units=metric")
            .with_status(404)
            .create();

        let result = fetch_weather("InvalidCity", "test_key").await;
        assert!(result.is_err());
    }
}