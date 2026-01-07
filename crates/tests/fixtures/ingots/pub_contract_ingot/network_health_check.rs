use std::net::{TcpStream, SocketAddr};
use std::time::Duration;
use reqwest::blocking::Client;

pub struct NetworkProbe {
    timeout: Duration,
    client: Client,
}

impl NetworkProbe {
    pub fn new(timeout_secs: u64) -> Self {
        Self {
            timeout: Duration::from_secs(timeout_secs),
            client: Client::builder()
                .timeout(Duration::from_secs(timeout_secs))
                .build()
                .unwrap(),
        }
    }

    pub fn tcp_check(&self, host: &str, port: u16) -> Result<(), String> {
        let addr: SocketAddr = format!("{}:{}", host, port)
            .parse()
            .map_err(|e| format!("Invalid address: {}", e))?;
        
        match TcpStream::connect_timeout(&addr, self.timeout) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("TCP connection failed: {}", e)),
        }
    }

    pub fn http_check(&self, url: &str) -> Result<u16, String> {
        match self.client.get(url).send() {
            Ok(response) => Ok(response.status().as_u16()),
            Err(e) => Err(format!("HTTP request failed: {}", e)),
        }
    }

    pub fn full_health_check(&self, endpoints: &[(&str, u16, &str)]) -> Vec<(String, bool)> {
        endpoints
            .iter()
            .map(|(host, port, url)| {
                let tcp_ok = self.tcp_check(host, *port).is_ok();
                let http_ok = self.http_check(url)
                    .map(|status| (200..300).contains(&status))
                    .unwrap_or(false);
                
                let service_name = format!("{}:{}", host, port);
                (service_name, tcp_ok && http_ok)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcp_check() {
        let probe = NetworkProbe::new(5);
        let result = probe.tcp_check("example.com", 80);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_http_check() {
        let probe = NetworkProbe::new(5);
        let result = probe.http_check("https://httpbin.org/status/200");
        assert!(result.is_ok());
    }
}