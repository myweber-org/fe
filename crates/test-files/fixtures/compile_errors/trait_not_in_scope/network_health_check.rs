
use std::net::{IpAddr, IcmpSocket, TcpStream};
use std::time::{Duration, Instant};
use std::thread;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(2);
const PING_COUNT: usize = 3;

pub struct NetworkCheckResult {
    pub host: String,
    pub icmp_reachable: bool,
    pub avg_ping_ms: Option<u128>,
    pub open_ports: Vec<u16>,
    pub check_duration: Duration,
}

pub fn check_host_connectivity(host: &str, ports: &[u16]) -> NetworkCheckResult {
    let start_time = Instant::now();
    let ip_addr: IpAddr = host.parse().unwrap_or_else(|_| {
        panic!("Invalid host address: {}", host)
    });

    let icmp_result = perform_icmp_check(&ip_addr);
    let port_result = scan_ports(&ip_addr, ports);

    NetworkCheckResult {
        host: host.to_string(),
        icmp_reachable: icmp_result.0,
        avg_ping_ms: icmp_result.1,
        open_ports: port_result,
        check_duration: start_time.elapsed(),
    }
}

fn perform_icmp_check(ip_addr: &IpAddr) -> (bool, Option<u128>) {
    let socket = match IcmpSocket::connect((*ip_addr, 0)) {
        Ok(s) => s,
        Err(_) => return (false, None),
    };

    let mut total_rtt = Duration::new(0, 0);
    let mut successful_pings = 0;

    for _ in 0..PING_COUNT {
        let ping_time = Instant::now();
        
        let payload = [0u8; 32];
        if socket.send(&payload).is_err() {
            continue;
        }

        let mut buffer = [0u8; 1024];
        socket.set_read_timeout(Some(DEFAULT_TIMEOUT)).ok();
        
        if socket.recv(&mut buffer).is_ok() {
            total_rtt += ping_time.elapsed();
            successful_pings += 1;
        }
        
        thread::sleep(Duration::from_millis(200));
    }

    if successful_pings > 0 {
        let avg_rtt = total_rtt.as_millis() / successful_pings as u128;
        (true, Some(avg_rtt))
    } else {
        (false, None)
    }
}

fn scan_ports(ip_addr: &IpAddr, ports: &[u16]) -> Vec<u16> {
    let mut open_ports = Vec::new();
    
    for &port in ports {
        if let Ok(_) = TcpStream::connect_timeout(
            &(*ip_addr, port).into(),
            DEFAULT_TIMEOUT
        ) {
            open_ports.push(port);
        }
    }
    
    open_ports
}

pub fn format_result(result: &NetworkCheckResult) -> String {
    let mut output = String::new();
    output.push_str(&format!("Host: {}\n", result.host));
    output.push_str(&format!("ICMP Reachable: {}\n", result.icmp_reachable));
    
    if let Some(ping) = result.avg_ping_ms {
        output.push_str(&format!("Average Ping: {} ms\n", ping));
    }
    
    if !result.open_ports.is_empty() {
        output.push_str(&format!("Open Ports: {:?}\n", result.open_ports));
    } else {
        output.push_str("No open ports found\n");
    }
    
    output.push_str(&format!("Check completed in: {:?}", result.check_duration));
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_localhost_connectivity() {
        let result = check_host_connectivity("127.0.0.1", &[80, 443, 8080]);
        assert_eq!(result.host, "127.0.0.1");
    }

    #[test]
    fn test_port_scanning() {
        let result = check_host_connectivity("8.8.8.8", &[53, 80, 443]);
        assert!(result.open_ports.len() <= 3);
    }
}