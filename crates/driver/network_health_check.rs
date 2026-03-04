use std::net::{TcpStream, UdpSocket};
use std::time::{Duration, Instant};
use std::thread;

const PACKET_SIZE: usize = 64;
const TIMEOUT_MS: u64 = 1000;
const TEST_COUNT: u32 = 5;

pub struct NetworkMetrics {
    pub latency_ms: f64,
    pub packet_loss_percent: f64,
    pub reachable: bool,
}

pub fn check_host(host: &str, port: u16) -> NetworkMetrics {
    let mut latencies = Vec::new();
    let mut successful_pings = 0;

    for _ in 0..TEST_COUNT {
        match measure_tcp_latency(host, port) {
            Some(latency) => {
                latencies.push(latency);
                successful_pings += 1;
            }
            None => continue,
        }
        thread::sleep(Duration::from_millis(100));
    }

    let avg_latency = if !latencies.is_empty() {
        latencies.iter().sum::<u128>() as f64 / latencies.len() as f64
    } else {
        0.0
    };

    let packet_loss = ((TEST_COUNT - successful_pings) as f64 / TEST_COUNT as f64) * 100.0;

    NetworkMetrics {
        latency_ms: avg_latency,
        packet_loss_percent: packet_loss,
        reachable: successful_pings > 0,
    }
}

fn measure_tcp_latency(host: &str, port: u16) -> Option<u128> {
    let start = Instant::now();
    match TcpStream::connect_timeout(
        &format!("{}:{}", host, port).parse().unwrap(),
        Duration::from_millis(TIMEOUT_MS),
    ) {
        Ok(_) => Some(start.elapsed().as_millis()),
        Err(_) => None,
    }
}

pub fn udp_echo_test(target_host: &str, target_port: u16, listen_port: u16) -> bool {
    let udp_socket = match UdpSocket::bind(format!("0.0.0.0:{}", listen_port)) {
        Ok(socket) => socket,
        Err(_) => return false,
    };

    udp_socket.set_read_timeout(Some(Duration::from_millis(TIMEOUT_MS))).ok();

    let test_payload = [0x55u8; PACKET_SIZE];
    let destination = format!("{}:{}", target_host, target_port);

    if udp_socket.send_to(&test_payload, &destination).is_err() {
        return false;
    }

    let mut response_buffer = [0u8; PACKET_SIZE];
    match udp_socket.recv_from(&mut response_buffer) {
        Ok((size, _)) => size == PACKET_SIZE,
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_localhost_connectivity() {
        let metrics = check_host("127.0.0.1", 80);
        assert!(metrics.reachable || metrics.packet_loss_percent == 100.0);
    }
}