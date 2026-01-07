
use reqwest;
use serde_json::Value;
use std::error::Error;

pub async fn fetch_weather(city: &str, api_key: &str) -> Result<Value, Box<dyn Error>> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );
    
    let response = reqwest::get(&url).await?;
    
    if !response.status().is_success() {
        return Err(format!("API request failed with status: {}", response.status()).into());
    }
    
    let weather_data: Value = response.json().await?;
    
    if weather_data.get("cod").and_then(|c| c.as_str()) != Some("200") {
        return Err(format!("Weather API error: {:?}", weather_data.get("message")).into());
    }
    
    Ok(weather_data)
}

pub fn extract_temperature(weather_data: &Value) -> Result<f64, Box<dyn Error>> {
    let temp = weather_data["main"]["temp"]
        .as_f64()
        .ok_or("Temperature data missing or invalid")?;
    
    Ok(temp)
}

pub fn extract_weather_description(weather_data: &Value) -> Result<String, Box<dyn Error>> {
    let description = weather_data["weather"][0]["description"]
        .as_str()
        .ok_or("Weather description missing")?
        .to_string();
    
    Ok(description)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extract_temperature() {
        let test_data = json!({
            "main": {
                "temp": 22.5
            }
        });
        
        let result = extract_temperature(&test_data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 22.5);
    }

    #[test]
    fn test_extract_weather_description() {
        let test_data = json!({
            "weather": [{
                "description": "clear sky"
            }]
        });
        
        let result = extract_weather_description(&test_data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "clear sky");
    }
}