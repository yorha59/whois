#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use rust_net_scanner_backend::{
    detect_network_internal, export_results_internal, perform_real_scan_internal, HostInfo, NetworkInfo
};

#[tauri::command]
fn detect_network() -> Result<NetworkInfo, String> {
    detect_network_internal()
}

#[tauri::command]
async fn perform_real_scan(subnet: String, extra_ports: Option<Vec<u16>>) -> Vec<HostInfo> {
    perform_real_scan_internal(subnet, extra_ports).await
}

#[tauri::command]
fn export_results(results: Vec<HostInfo>, format: String) -> Result<String, String> {
    export_results_internal(results, format)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            detect_network,
            perform_real_scan,
            export_results,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
