use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::time::Duration;

#[derive(Debug)]
pub struct NetworkCheckResult {
    pub host: String,
    pub ping_success: bool,
    pub open_ports: Vec<u16>,
}

pub fn check_network_health(host: &str, ports: &[u16]) -> NetworkCheckResult {
    let ping_result = ping_host(host);
    let open_ports = scan_ports(host, ports);
    
    NetworkCheckResult {
        host: host.to_string(),
        ping_success: ping_result,
        open_ports,
    }
}

fn ping_host(host: &str) -> bool {
    use std::process::Command;
    
    let output = if cfg!(target_os = "windows") {
        Command::new("ping")
            .args(["-n", "1", "-w", "1000", host])
            .output()
    } else {
        Command::new("ping")
            .args(["-c", "1", "-W", "1", host])
            .output()
    };
    
    match output {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

fn scan_ports(host: &str, ports: &[u16]) -> Vec<u16> {
    let mut open_ports = Vec::new();
    
    for &port in ports {
        let socket_addr = SocketAddr::new(
            host.parse().unwrap_or(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
            port,
        );
        
        if TcpStream::connect_timeout(&socket_addr, Duration::from_secs(1)).is_ok() {
            open_ports.push(port);
        }
    }
    
    open_ports
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_localhost_scan() {
        let result = check_network_health("127.0.0.1", &[80, 443, 8080]);
        assert_eq!(result.host, "127.0.0.1");
        // At least ping should work for localhost
        assert!(result.ping_success);
    }
}