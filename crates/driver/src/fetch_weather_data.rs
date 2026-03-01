use std::collections::HashMap;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone)]
struct WeatherData {
    temperature: f64,
    humidity: f64,
    description: String,
    timestamp: SystemTime,
}

struct WeatherCache {
    cache: HashMap<String, (WeatherData, SystemTime)>,
    ttl: Duration,
}

impl WeatherCache {
    fn new(ttl_seconds: u64) -> Self {
        WeatherCache {
            cache: HashMap::new(),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    fn get(&mut self, city: &str) -> Option<WeatherData> {
        let now = SystemTime::now();
        if let Some((data, timestamp)) = self.cache.get(city) {
            if now.duration_since(*timestamp).unwrap() < self.ttl {
                return Some(data.clone());
            }
            self.cache.remove(city);
        }
        None
    }

    fn insert(&mut self, city: String, data: WeatherData) {
        self.cache.insert(city, (data, SystemTime::now()));
    }
}

fn fetch_weather_from_api(city: &str) -> Result<WeatherData, String> {
    // Simulate API call
    if city.is_empty() {
        return Err("City name cannot be empty".to_string());
    }

    Ok(WeatherData {
        temperature: 22.5,
        humidity: 65.0,
        description: "Partly cloudy".to_string(),
        timestamp: SystemTime::now(),
    })
}

pub fn get_weather(city: &str, cache: &mut WeatherCache) -> Result<WeatherData, String> {
    if let Some(cached_data) = cache.get(city) {
        return Ok(cached_data);
    }

    let weather_data = fetch_weather_from_api(city)?;
    cache.insert(city.to_string(), weather_data.clone());
    Ok(weather_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_miss_then_hit() {
        let mut cache = WeatherCache::new(300);
        let city = "London";

        // First call should fetch from API
        let result1 = get_weather(city, &mut cache);
        assert!(result1.is_ok());

        // Second call should return cached data
        let result2 = get_weather(city, &mut cache);
        assert!(result2.is_ok());

        // Verify both results are equal
        let data1 = result1.unwrap();
        let data2 = result2.unwrap();
        assert_eq!(data1.temperature, data2.temperature);
        assert_eq!(data1.description, data2.description);
    }

    #[test]
    fn test_cache_expiration() {
        let mut cache = WeatherCache::new(1); // 1 second TTL
        let city = "Paris";

        // First call caches data
        let _ = get_weather(city, &mut cache);

        // Wait for cache to expire
        std::thread::sleep(Duration::from_secs(2));

        // Should fetch fresh data
        let result = get_weather(city, &mut cache);
        assert!(result.is_ok());
    }
}