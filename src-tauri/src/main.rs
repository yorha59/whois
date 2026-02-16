
#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};
use std::net::{IpAddr, SocketAddr};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceType {
    SSH, HTTP, HTTPS, FTP, Redis, PostgreSQL, MySQL, Unknown,
}

impl ServiceType {
    pub fn from_port(port: u16) -> Self {
        match port {
            21 => ServiceType::FTP,
            22 => ServiceType.SSH,
            80 => ServiceType.HTTP,
            443 => ServiceType.HTTPS,
            3306 => ServiceType.MySQL,
            5432 => ServiceType.PostgreSQL,
            6379 => ServiceType.Redis,
            _ => ServiceType.Unknown,
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
    let timeout_duration = Duration::from_millis(300);

    match timeout(timeout_duration, TcpStream::connect(&addr)).await {
        Ok(Ok(_)) => Some(PortInfo {
            port,
            service: ServiceType::from_port(port),
        }),
        _ => None,
    }
}

#[tauri::command]
async fn perform_real_scan(subnet: String) -> Vec<HostInfo> {
    let mut results = Vec::new();
    let common_ports = vec![21, 22, 80, 443, 3306, 5432, 6379, 8080];
    let mut handles = Vec::new();

    for i in 1..255 {
        let ip_str = format!("{}.{}", subnet, i);
        let ip: IpAddr = ip_str.parse().unwrap();
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
                    hostname: None, 
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

fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![perform_real_scan])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
