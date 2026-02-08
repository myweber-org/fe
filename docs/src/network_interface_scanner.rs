use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::process::Command;
use std::str;

#[derive(Debug)]
pub struct NetworkInterface {
    pub name: String,
    pub ipv4_addresses: Vec<Ipv4Addr>,
    pub ipv6_addresses: Vec<Ipv6Addr>,
    pub mac_address: Option<String>,
    pub is_up: bool,
}

impl NetworkInterface {
    pub fn new(name: String) -> Self {
        NetworkInterface {
            name,
            ipv4_addresses: Vec::new(),
            ipv6_addresses: Vec::new(),
            mac_address: None,
            is_up: false,
        }
    }
}

pub fn scan_interfaces() -> Result<Vec<NetworkInterface>, String> {
    let mut interfaces = Vec::new();
    
    match Command::new("ip").arg("addr").output() {
        Ok(output) => {
            let output_str = str::from_utf8(&output.stdout)
                .map_err(|e| format!("Failed to parse ip output: {}", e))?;
            
            let mut current_interface: Option<NetworkInterface> = None;
            
            for line in output_str.lines() {
                if line.trim().is_empty() {
                    continue;
                }
                
                if line.starts_with(|c: char| c.is_numeric()) {
                    if let Some(iface) = current_interface.take() {
                        interfaces.push(iface);
                    }
                    
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let name = parts[1].trim_end_matches(':').to_string();
                        current_interface = Some(NetworkInterface::new(name));
                    }
                } else if let Some(ref mut iface) = current_interface {
                    let trimmed = line.trim();
                    
                    if trimmed.starts_with("link/") {
                        let parts: Vec<&str> = trimmed.split_whitespace().collect();
                        if parts.len() >= 2 {
                            iface.mac_address = Some(parts[1].to_string());
                        }
                    } else if trimmed.starts_with("inet ") {
                        let parts: Vec<&str> = trimmed.split_whitespace().collect();
                        if parts.len() >= 2 {
                            if let Ok(ip) = parts[1].split('/').next().unwrap().parse::<Ipv4Addr>() {
                                iface.ipv4_addresses.push(ip);
                            }
                        }
                    } else if trimmed.starts_with("inet6 ") {
                        let parts: Vec<&str> = trimmed.split_whitespace().collect();
                        if parts.len() >= 2 {
                            if let Ok(ip) = parts[1].split('/').next().unwrap().parse::<Ipv6Addr>() {
                                iface.ipv6_addresses.push(ip);
                            }
                        }
                    } else if trimmed.contains("state UP") {
                        iface.is_up = true;
                    }
                }
            }
            
            if let Some(iface) = current_interface.take() {
                interfaces.push(iface);
            }
            
            Ok(interfaces)
        }
        Err(_) => {
            eprintln!("'ip' command not found, trying 'ifconfig' as fallback");
            fallback_scan()
        }
    }
}

fn fallback_scan() -> Result<Vec<NetworkInterface>, String> {
    let output = Command::new("ifconfig")
        .output()
        .map_err(|e| format!("Failed to execute ifconfig: {}", e))?;
    
    let output_str = str::from_utf8(&output.stdout)
        .map_err(|e| format!("Failed to parse ifconfig output: {}", e))?;
    
    let mut interfaces = Vec::new();
    let mut current_interface: Option<NetworkInterface> = None;
    
    for line in output_str.lines() {
        if !line.starts_with(' ') && !line.starts_with('\t') && !line.is_empty() {
            if let Some(iface) = current_interface.take() {
                interfaces.push(iface);
            }
            
            let name = line.split_whitespace().next().unwrap_or("").to_string();
            current_interface = Some(NetworkInterface::new(name));
        } else if let Some(ref mut iface) = current_interface {
            let trimmed = line.trim();
            
            if trimmed.starts_with("ether ") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 2 {
                    iface.mac_address = Some(parts[1].to_string());
                }
            } else if trimmed.starts_with("inet ") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(ip) = parts[1].parse::<Ipv4Addr>() {
                        iface.ipv4_addresses.push(ip);
                    }
                }
            } else if trimmed.starts_with("inet6 ") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(ip) = parts[1].parse::<Ipv6Addr>() {
                        iface.ipv6_addresses.push(ip);
                    }
                }
            } else if trimmed.contains("status: active") {
                iface.is_up = true;
            }
        }
    }
    
    if let Some(iface) = current_interface.take() {
        interfaces.push(iface);
    }
    
    Ok(interfaces)
}

pub fn display_interfaces(interfaces: &[NetworkInterface]) {
    println!("Available Network Interfaces:");
    println!("{:-<60}", "");
    
    for iface in interfaces {
        println!("Interface: {}", iface.name);
        println!("  Status: {}", if iface.is_up { "UP" } else { "DOWN" });
        
        if let Some(ref mac) = iface.mac_address {
            println!("  MAC Address: {}", mac);
        }
        
        if !iface.ipv4_addresses.is_empty() {
            println!("  IPv4 Addresses:");
            for addr in &iface.ipv4_addresses {
                println!("    - {}", addr);
            }
        }
        
        if !iface.ipv6_addresses.is_empty() {
            println!("  IPv6 Addresses:");
            for addr in &iface.ipv6_addresses {
                println!("    - {}", addr);
            }
        }
        
        println!("{:-<60}", "");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_interface_creation() {
        let iface = NetworkInterface::new("eth0".to_string());
        assert_eq!(iface.name, "eth0");
        assert!(iface.ipv4_addresses.is_empty());
        assert!(iface.ipv6_addresses.is_empty());
        assert!(iface.mac_address.is_none());
        assert!(!iface.is_up);
    }
    
    #[test]
    fn test_scan_interfaces() {
        let result = scan_interfaces();
        assert!(result.is_ok(), "Failed to scan interfaces: {:?}", result.err());
        
        let interfaces = result.unwrap();
        assert!(!interfaces.is_empty(), "No network interfaces found");
        
        for iface in interfaces {
            assert!(!iface.name.is_empty());
        }
    }
}