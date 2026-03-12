use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::time::Duration;

#[derive(Debug)]
struct HostCheck {
    host: String,
    port: u16,
    timeout_secs: u64,
}

impl HostCheck {
    fn new(host: &str, port: u16, timeout_secs: u64) -> Self {
        HostCheck {
            host: host.to_string(),
            port,
            timeout_secs,
        }
    }

    fn check(&self) -> Result<(), String> {
        let socket_addr = SocketAddr::new(
            match self.host.parse::<IpAddr>() {
                Ok(ip) => ip,
                Err(_) => IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            },
            self.port,
        );

        match TcpStream::connect_timeout(
            &socket_addr,
            Duration::from_secs(self.timeout_secs),
        ) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to connect to {}:{} - {}", self.host, self.port, e)),
        }
    }
}

fn main() {
    let hosts_to_check = vec![
        HostCheck::new("8.8.8.8", 53, 2),
        HostCheck::new("1.1.1.1", 53, 2),
        HostCheck::new("localhost", 8080, 1),
    ];

    println!("Starting network health check...");
    for host in hosts_to_check {
        print!("Checking {}:{}... ", host.host, host.port);
        match host.check() {
            Ok(()) => println!("OK"),
            Err(e) => println!("FAILED - {}", e),
        }
    }
    println!("Health check complete.");
}