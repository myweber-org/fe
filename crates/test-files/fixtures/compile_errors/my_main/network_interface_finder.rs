use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct NetworkInterface {
    pub name: String,
    pub ipv4_addresses: Vec<Ipv4Addr>,
    pub ipv6_addresses: Vec<Ipv6Addr>,
    pub mac_address: Option<[u8; 6]>,
    pub is_up: bool,
}

pub fn get_network_interfaces() -> Result<Vec<NetworkInterface>, String> {
    let mut interfaces = Vec::new();
    
    match pnet_datalink::interfaces() {
        Ok(net_interfaces) => {
            for iface in net_interfaces {
                let mut ipv4_addrs = Vec::new();
                let mut ipv6_addrs = Vec::new();
                
                for ip_network in iface.ips {
                    match ip_network.ip() {
                        IpAddr::V4(ipv4) => ipv4_addrs.push(ipv4),
                        IpAddr::V6(ipv6) => ipv6_addrs.push(ipv6),
                    }
                }
                
                let mac_address = if let Some(mac) = iface.mac {
                    Some(mac.octets())
                } else {
                    None
                };
                
                interfaces.push(NetworkInterface {
                    name: iface.name.clone(),
                    ipv4_addresses: ipv4_addrs,
                    ipv6_addresses: ipv6_addrs,
                    mac_address,
                    is_up: iface.is_up(),
                });
            }
            Ok(interfaces)
        }
        Err(e) => Err(format!("Failed to get network interfaces: {}", e)),
    }
}

pub fn find_interface_by_ip(ip: IpAddr) -> Option<NetworkInterface> {
    match get_network_interfaces() {
        Ok(interfaces) => {
            for iface in interfaces {
                match ip {
                    IpAddr::V4(ipv4) => {
                        if iface.ipv4_addresses.contains(&ipv4) {
                            return Some(iface);
                        }
                    }
                    IpAddr::V6(ipv6) => {
                        if iface.ipv6_addresses.contains(&ipv6) {
                            return Some(iface);
                        }
                    }
                }
            }
            None
        }
        Err(_) => None,
    }
}

pub fn get_interface_map() -> HashMap<String, NetworkInterface> {
    let mut map = HashMap::new();
    
    if let Ok(interfaces) = get_network_interfaces() {
        for iface in interfaces {
            map.insert(iface.name.clone(), iface);
        }
    }
    
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_get_interfaces() {
        let interfaces = get_network_interfaces();
        assert!(interfaces.is_ok());
        
        if let Ok(ifaces) = interfaces {
            assert!(!ifaces.is_empty());
            
            for iface in ifaces {
                assert!(!iface.name.is_empty());
                println!("Interface: {}, Up: {}", iface.name, iface.is_up);
            }
        }
    }
    
    #[test]
    fn test_interface_map() {
        let map = get_interface_map();
        assert!(!map.is_empty());
    }
}