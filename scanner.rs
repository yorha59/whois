
/*
 * Rust Network Scanner (Tauri Compatible Version)
 * 适配 Tauri 命令系统，可以直接在 Tauri 的 main.rs 中引用
 */

use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceType {
    SSH, HTTP, HTTPS, FTP, MySQL, Postgres, Redis, Unknown,
}

impl ServiceType {
    pub fn from_port(port: u16) -> Self {
        match port {
            21 => ServiceType::FTP,
            22 => ServiceType::SSH,
            80 => ServiceType::HTTP,
            443 => ServiceType::HTTPS,
            3306 => ServiceType::MySQL,
            5432 => ServiceType::Postgres,
            6379 => ServiceType::Redis,
            _ => ServiceType::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortInfo {
    pub port: u16,
    pub service: ServiceType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostInfo {
    pub ip: String,
    pub hostname: Option<String>,
    pub ports: Vec<PortInfo>,
}

async fn scan_port(ip: IpAddr, port: u16) -> Option<PortInfo> {
    let addr = SocketAddr::new(ip, port);
    let timeout_duration = Duration::from_millis(400); // 局域网更快的超时

    match timeout(timeout_duration, TcpStream::connect(&addr)).await {
        Ok(Ok(_)) => Some(PortInfo {
            port,
            service: ServiceType::from_port(port),
        }),
        _ => None,
    }
}

// 暴露给 Tauri 前端的命令
#[tauri::command]
pub async fn perform_scan(subnet_prefix: String) -> Vec<HostInfo> {
    let mut results = Vec::new();
    let common_ports = vec![21, 22, 80, 443, 3306, 5432, 6379, 8080];
    let mut handles = Vec::new();

    // 扫描 1-254
    for i in 1..255 {
        let ip_str = format!("{}.{}", subnet_prefix, i);
        let ip: IpAddr = ip_str.parse().expect("Invalid IP");
        let ports = common_ports.clone();
        
        handles.push(tokio::spawn(async move {
            let mut found_ports = Vec::new();
            for port in ports {
                if let Some(info) = scan_port(ip, port).await {
                    found_ports.push(info);
                }
            }
            if !found_ports.is_empty() {
                Some(HostInfo {
                    ip: ip_str,
                    hostname: None, // 实际应用中可结合 mDNS 库解析
                    ports: found_ports,
                })
            } else {
                None
            }
        }));
    }

    for handle in handles {
        if let Ok(Some(host)) = handle.await {
            results.push(host);
        }
    }

    results
}
