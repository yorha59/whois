#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use rust_net_scanner_backend::{
    detect_network_internal, export_results_internal, perform_real_scan_internal,
    HostInfo, NetworkInfo, ScanConfig,
};
use std::env;

#[tauri::command]
fn detect_network() -> Result<NetworkInfo, String> {
    detect_network_internal()
}

#[tauri::command]
async fn perform_real_scan(subnet: String, extra_ports: Option<Vec<u16>>) -> Vec<HostInfo> {
    perform_real_scan_internal(subnet, extra_ports, None).await
}

#[tauri::command]
fn export_results(results: Vec<HostInfo>, format: String) -> Result<String, String> {
    export_results_internal(results, format)
}

/// CLI scanning mode - prints results as a formatted table
async fn run_cli_scan(subnet: Option<String>, timeout_ms: Option<u64>) {
    let network_info = match subnet {
        Some(s) => NetworkInfo {
            local_ip: format!("{}.1", s),
            subnet: s,
        },
        None => match detect_network_internal() {
            Ok(info) => {
                println!("Detected subnet: {}/24", info.subnet);
                info
            }
            Err(e) => {
                eprintln!("Error detecting network: {}", e);
                eprintln!("Usage: {} --scan --subnet 192.168.1", env::args().next().unwrap());
                std::process::exit(1);
            }
        },
    };

    let config = ScanConfig {
        timeout_ms: timeout_ms.unwrap_or(1000),
        max_concurrent: 100,
    };

    println!(
        "Scanning {}/24 (timeout: {}ms, max concurrent: {})...",
        network_info.subnet, config.timeout_ms, config.max_concurrent
    );
    println!();

    let results = perform_real_scan_internal(network_info.subnet, None, Some(config)).await;

    if results.is_empty() {
        println!("No hosts found.");
    } else {
        // Print table header
        println!("{:<16} {:<30} {}", "IP Address", "Hostname", "Ports");
        println!("{}", "-".repeat(90));

        // Print results
        for host in &results {
            let hostname = host.hostname.as_deref().unwrap_or("-");
            let ports: Vec<String> = host.ports.iter()
                .map(|p| format!("{} ({})", p.port, p.service_label))
                .collect();
            let ports_str = if ports.is_empty() { "-".to_string() } else { ports.join(", ") };
            println!("{:<16} {:<30} {}", host.ip, hostname, ports_str);
        }

        // Print service discovery summary
        let hosts_with_mdns: Vec<_> = results.iter()
            .filter(|h| h.mdns_services.as_ref().map(|s| !s.is_empty()).unwrap_or(false))
            .collect();
        let hosts_with_ssdp: Vec<_> = results.iter()
            .filter(|h| h.ssdp_devices.as_ref().map(|d| !d.is_empty()).unwrap_or(false))
            .collect();

        if !hosts_with_mdns.is_empty() || !hosts_with_ssdp.is_empty() {
            println!();
            println!("Service Discovery Results:");
            println!("{}", "-".repeat(90));

            for host in &hosts_with_mdns {
                if let Some(services) = &host.mdns_services {
                    for svc in services {
                        println!("  [mDNS] {} - {} ({})", host.ip, svc.service_type, svc.name);
                    }
                }
            }

            for host in &hosts_with_ssdp {
                if let Some(devices) = &host.ssdp_devices {
                    for dev in devices {
                        let device_name = if dev.device_type.len() > 40 {
                            format!("{}...", &dev.device_type[..37])
                        } else {
                            dev.device_type.clone()
                        };
                        println!("  [SSDP] {} - {}", host.ip, device_name);
                        if let Some(loc) = &dev.location {
                            println!("         Location: {}", loc);
                        }
                    }
                }
            }
        }
    }
}

fn print_help(program_name: &str) {
    println!("Network Scanner - CLI Mode");
    println!();
    println!("Usage: {} [OPTIONS]", program_name);
    println!();
    println!("Options:");
    println!("  --scan             Run scan in CLI mode");
    println!("  --subnet <subnet>  Specify subnet (e.g., 192.168.1)");
    println!("  --timeout <ms>     Set port timeout in milliseconds (default: 1000)");
    println!("  --help             Show this help message");
    println!();
    println!("Examples:");
    println!("  {} --scan", program_name);
    println!("  {} --scan --subnet 192.168.1", program_name);
    println!("  {} --scan --subnet 10.0.0 --timeout 2000", program_name);
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let program_name = args[0].clone();

    // Check for CLI mode
    let mut scan_mode = false;
    let mut subnet = None;
    let mut timeout_ms = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--scan" => scan_mode = true,
            "--subnet" => {
                if i + 1 < args.len() {
                    subnet = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--timeout" => {
                if i + 1 < args.len() {
                    if let Ok(t) = args[i + 1].parse::<u64>() {
                        timeout_ms = Some(t);
                    }
                    i += 1;
                }
            }
            "--help" | "-h" => {
                print_help(&program_name);
                return;
            }
            _ => {}
        }
        i += 1;
    }

    if scan_mode {
        // CLI scanning mode
        run_cli_scan(subnet, timeout_ms).await;
    } else {
        // Tauri GUI mode
        tauri::Builder::default()
            .invoke_handler(tauri::generate_handler![
                detect_network,
                perform_real_scan,
                export_results,
            ])
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    }
}
