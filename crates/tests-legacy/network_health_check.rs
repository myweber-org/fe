use std::net::{TcpStream, UdpSocket};
use std::time::{Duration, Instant};
use std::thread;

const PACKET_SIZE: usize = 64;
const TIMEOUT_SECS: u64 = 5;
const SAMPLE_COUNT: usize = 10;

struct NetworkMetrics {
    latency_ms: f64,
    packet_loss_percent: f64,
    jitter_ms: f64,
}

fn measure_tcp_latency(host: &str, port: u16) -> Option<Duration> {
    let start = Instant::now();
    match TcpStream::connect_timeout(
        &format!("{}:{}", host, port).parse().unwrap(),
        Duration::from_secs(TIMEOUT_SECS)
    ) {
        Ok(_) => Some(start.elapsed()),
        Err(_) => None,
    }
}

fn measure_udp_packet_loss(host: &str, port: u16) -> (usize, usize) {
    let socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(_) => return (0, 0),
    };
    
    socket.set_read_timeout(Some(Duration::from_millis(500))).unwrap();
    
    let mut received = 0;
    let test_data = vec![0u8; PACKET_SIZE];
    
    for _ in 0..SAMPLE_COUNT {
        if socket.send_to(&test_data, (host, port)).is_ok() {
            let mut buffer = [0u8; PACKET_SIZE];
            if socket.recv_from(&mut buffer).is_ok() {
                received += 1;
            }
        }
        thread::sleep(Duration::from_millis(100));
    }
    
    (received, SAMPLE_COUNT)
}

fn calculate_jitter(latencies: &[Duration]) -> f64 {
    if latencies.len() < 2 {
        return 0.0;
    }
    
    let mut diffs = Vec::new();
    for i in 1..latencies.len() {
        let diff = (latencies[i].as_micros() as i128 - latencies[i-1].as_micros() as i128).abs();
        diffs.push(diff as f64 / 1000.0);
    }
    
    diffs.iter().sum::<f64>() / diffs.len() as f64
}

pub fn check_network_health(host: &str, tcp_port: u16, udp_port: u16) -> NetworkMetrics {
    let mut latencies = Vec::new();
    let mut successful_pings = 0;
    
    for _ in 0..SAMPLE_COUNT {
        if let Some(latency) = measure_tcp_latency(host, tcp_port) {
            latencies.push(latency);
            successful_pings += 1;
        }
        thread::sleep(Duration::from_millis(200));
    }
    
    let (received, sent) = measure_udp_packet_loss(host, udp_port);
    
    let avg_latency = if !latencies.is_empty() {
        latencies.iter().map(|d| d.as_micros() as f64).sum::<f64>() / latencies.len() as f64 / 1000.0
    } else {
        0.0
    };
    
    let packet_loss = if sent > 0 {
        (1.0 - received as f64 / sent as f64) * 100.0
    } else {
        100.0
    };
    
    let jitter = calculate_jitter(&latencies);
    
    NetworkMetrics {
        latency_ms: avg_latency,
        packet_loss_percent: packet_loss,
        jitter_ms: jitter,
    }
}

pub fn print_metrics(metrics: &NetworkMetrics) {
    println!("Network Health Report:");
    println!("  Average Latency: {:.2} ms", metrics.latency_ms);
    println!("  Packet Loss: {:.1}%", metrics.packet_loss_percent);
    println!("  Jitter: {:.2} ms", metrics.jitter_ms);
    
    if metrics.latency_ms < 50.0 && metrics.packet_loss_percent < 1.0 {
        println!("  Status: Excellent");
    } else if metrics.latency_ms < 100.0 && metrics.packet_loss_percent < 5.0 {
        println!("  Status: Good");
    } else if metrics.latency_ms < 200.0 && metrics.packet_loss_percent < 10.0 {
        println!("  Status: Fair");
    } else {
        println!("  Status: Poor");
    }
}