use std::net::{TcpStream, SocketAddr};
use std::time::Duration;
use std::io;

pub struct NetworkProbe {
    timeout: Duration,
}

impl NetworkProbe {
    pub fn new(timeout_secs: u64) -> Self {
        NetworkProbe {
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    pub fn check_port(&self, host: &str, port: u16) -> io::Result<bool> {
        let addr: SocketAddr = format!("{}:{}", host, port).parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        
        match TcpStream::connect_timeout(&addr, self.timeout) {
            Ok(_) => Ok(true),
            Err(e) if e.kind() == io::ErrorKind::TimedOut => Ok(false),
            Err(e) => Err(e),
        }
    }

    pub fn scan_ports(&self, host: &str, start_port: u16, end_port: u16) -> Vec<u16> {
        (start_port..=end_port)
            .filter(|&port| self.check_port(host, port).unwrap_or(false))
            .collect()
    }
}

pub fn validate_hostname(hostname: &str) -> bool {
    !hostname.is_empty() && 
    hostname.len() <= 253 &&
    hostname.split('.').all(|part| {
        !part.is_empty() && 
        part.len() <= 63 &&
        part.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') &&
        !part.starts_with('-') && 
        !part.ends_with('-')
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_check_localhost() {
        let probe = NetworkProbe::new(2);
        let result = probe.check_port("127.0.0.1", 80);
        assert!(result.is_ok());
    }

    #[test]
    fn test_hostname_validation() {
        assert!(validate_hostname("example.com"));
        assert!(validate_hostname("sub.domain.co.uk"));
        assert!(!validate_hostname("-invalid.com"));
        assert!(!validate_hostname(""));
    }
}