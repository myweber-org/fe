
use std::net::{TcpStream, SocketAddr};
use std::time::{Duration, Instant};
use std::io::{self, Write};

pub struct NetworkHealthChecker {
    target_hosts: Vec<String>,
    timeout: Duration,
}

impl NetworkHealthChecker {
    pub fn new(hosts: Vec<String>) -> Self {
        NetworkHealthChecker {
            target_hosts: hosts,
            timeout: Duration::from_secs(5),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn check_all(&self) -> Vec<HealthResult> {
        self.target_hosts
            .iter()
            .map(|host| self.check_host(host))
            .collect()
    }

    fn check_host(&self, host: &str) -> HealthResult {
        let start = Instant::now();
        let port = 80;
        let socket_addr = format!("{}:{}", host, port);
        
        match socket_addr.parse::<SocketAddr>() {
            Ok(addr) => {
                match TcpStream::connect_timeout(&addr, self.timeout) {
                    Ok(_) => {
                        let latency = start.elapsed();
                        HealthResult::success(host, latency)
                    }
                    Err(e) => HealthResult::failure(host, &e.to_string()),
                }
            }
            Err(e) => HealthResult::failure(host, &e.to_string()),
        }
    }

    pub fn generate_report(&self) -> String {
        let results = self.check_all();
        let mut report = String::new();
        report.push_str("Network Health Check Report\n");
        report.push_str("===========================\n\n");

        for result in results {
            report.push_str(&format!("Host: {}\n", result.host));
            report.push_str(&format!("Status: {}\n", result.status));
            if let Some(latency) = result.latency {
                report.push_str(&format!("Latency: {:?}\n", latency));
            }
            if let Some(error) = &result.error {
                report.push_str(&format!("Error: {}\n", error));
            }
            report.push_str("\n");
        }

        report
    }
}

#[derive(Debug, Clone)]
pub struct HealthResult {
    pub host: String,
    pub status: HealthStatus,
    pub latency: Option<Duration>,
    pub error: Option<String>,
}

impl HealthResult {
    fn success(host: &str, latency: Duration) -> Self {
        HealthResult {
            host: host.to_string(),
            status: HealthStatus::Healthy,
            latency: Some(latency),
            error: None,
        }
    }

    fn failure(host: &str, error: &str) -> Self {
        HealthResult {
            host: host.to_string(),
            status: HealthStatus::Unhealthy,
            latency: None,
            error: Some(error.to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "Healthy"),
            HealthStatus::Unhealthy => write!(f, "Unhealthy"),
        }
    }
}

pub fn run_health_check() -> io::Result<()> {
    let hosts = vec![
        "google.com".to_string(),
        "github.com".to_string(),
        "rust-lang.org".to_string(),
    ];

    let checker = NetworkHealthChecker::new(hosts)
        .with_timeout(Duration::from_secs(3));

    let report = checker.generate_report();
    io::stdout().write_all(report.as_bytes())?;
    
    Ok(())
}