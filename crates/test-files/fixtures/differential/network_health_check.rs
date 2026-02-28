
use std::time::{Duration, Instant};
use std::net::{IpAddr, IcmpSocket, SocketAddr};
use rand::Rng;

const PACKET_SIZE: usize = 64;
const TIMEOUT: Duration = Duration::from_secs(2);
const MAX_PACKETS: usize = 10;

pub struct NetworkHealth {
    target: IpAddr,
    packet_loss: f32,
    avg_latency: Duration,
    jitter: Duration,
}

impl NetworkHealth {
    pub fn new(target: IpAddr) -> Self {
        NetworkHealth {
            target,
            packet_loss: 0.0,
            avg_latency: Duration::from_millis(0),
            jitter: Duration::from_millis(0),
        }
    }

    pub fn perform_check(&mut self) -> Result<(), String> {
        let mut latencies = Vec::new();
        let mut lost_packets = 0;

        for seq in 0..MAX_PACKETS {
            match self.send_icmp_packet(seq) {
                Ok(latency) => latencies.push(latency),
                Err(_) => lost_packets += 1,
            }
            std::thread::sleep(Duration::from_millis(100));
        }

        if latencies.is_empty() {
            return Err("All packets lost".to_string());
        }

        self.packet_loss = (lost_packets as f32 / MAX_PACKETS as f32) * 100.0;
        self.avg_latency = self.calculate_average(&latencies);
        self.jitter = self.calculate_jitter(&latencies);

        Ok(())
    }

    fn send_icmp_packet(&self, sequence: usize) -> Result<Duration, String> {
        let socket = IcmpSocket::bind("0.0.0.0:0")
            .map_err(|e| format!("Failed to bind socket: {}", e))?;

        socket.set_read_timeout(Some(TIMEOUT))
            .map_err(|e| format!("Failed to set timeout: {}", e))?;

        let mut packet = [0u8; PACKET_SIZE];
        let mut rng = rand::thread_rng();
        rng.fill(&mut packet[..]);

        let start = Instant::now();
        
        socket.send_to(&packet, SocketAddr::new(self.target, 0))
            .map_err(|e| format!("Failed to send packet: {}", e))?;

        let mut buffer = [0u8; 1024];
        socket.recv_from(&mut buffer)
            .map_err(|_| "Packet receive timeout".to_string())?;

        Ok(start.elapsed())
    }

    fn calculate_average(&self, latencies: &[Duration]) -> Duration {
        let total: Duration = latencies.iter().sum();
        total / latencies.len() as u32
    }

    fn calculate_jitter(&self, latencies: &[Duration]) -> Duration {
        if latencies.len() < 2 {
            return Duration::from_millis(0);
        }

        let mut diffs = Vec::new();
        for i in 1..latencies.len() {
            let diff = latencies[i].as_micros() as i128 - latencies[i-1].as_micros() as i128;
            diffs.push(diff.abs() as u128);
        }

        let avg_diff: u128 = diffs.iter().sum::<u128>() / diffs.len() as u128;
        Duration::from_micros(avg_diff as u64)
    }

    pub fn generate_report(&self) -> String {
        format!(
            "Network Health Report for {}:\n\
             Packet Loss: {:.1}%\n\
             Average Latency: {:.2}ms\n\
             Jitter: {:.2}ms",
            self.target,
            self.packet_loss,
            self.avg_latency.as_micros() as f32 / 1000.0,
            self.jitter.as_micros() as f32 / 1000.0
        )
    }
}

pub fn check_connectivity(host: &str) -> Result<NetworkHealth, String> {
    let addr: IpAddr = host.parse()
        .map_err(|e| format!("Invalid address: {}", e))?;

    let mut checker = NetworkHealth::new(addr);
    checker.perform_check()?;
    
    Ok(checker)
}