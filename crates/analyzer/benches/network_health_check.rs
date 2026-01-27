use std::net::TcpStream;
use std::time::Duration;

pub struct HealthCheck {
    host: String,
    port: u16,
    timeout: Duration,
    max_retries: u32,
}

impl HealthCheck {
    pub fn new(host: String, port: u16) -> Self {
        HealthCheck {
            host,
            port,
            timeout: Duration::from_secs(5),
            max_retries: 3,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    pub fn check(&self) -> Result<(), String> {
        let address = format!("{}:{}", self.host, self.port);
        
        for attempt in 1..=self.max_retries {
            match TcpStream::connect_timeout(&address.parse().unwrap(), self.timeout) {
                Ok(_) => return Ok(()),
                Err(e) => {
                    if attempt == self.max_retries {
                        return Err(format!("Failed after {} attempts: {}", self.max_retries, e));
                    }
                    std::thread::sleep(Duration::from_secs(1));
                }
            }
        }
        
        Err("Max retries exceeded".to_string())
    }
}

pub fn quick_check(host: &str, port: u16) -> bool {
    let checker = HealthCheck::new(host.to_string(), port);
    checker.check().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check_creation() {
        let checker = HealthCheck::new("localhost".to_string(), 8080);
        assert_eq!(checker.host, "localhost");
        assert_eq!(checker.port, 8080);
    }

    #[test]
    fn test_with_custom_config() {
        let checker = HealthCheck::new("example.com".to_string(), 443)
            .with_timeout(Duration::from_secs(10))
            .with_max_retries(5);
        
        assert_eq!(checker.timeout.as_secs(), 10);
        assert_eq!(checker.max_retries, 5);
    }
}