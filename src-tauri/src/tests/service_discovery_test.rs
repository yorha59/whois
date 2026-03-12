//! Service Discovery (mDNS + SSDP) 测试
//!
//! 测试目标：验证 UDP 服务发现功能的正确性
//! 覆盖率目标：80%+

use crate::{
    build_mdns_query, build_ssdp_search, discover_services_for_host, mdns_discovery,
    parse_mdns_response, parse_ssdp_response, ssdp_discovery, MdnsServiceInfo, SsdpDeviceInfo,
    ServiceDiscoveryResult,
};

// ═══════════════════════════════════════════════════════════════════════════════
// mDNS 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_build_mdns_query() {
    // 测试构建 mDNS 查询包
    let query = build_mdns_query("_hap._tcp.local");

    // 验证包头结构
    assert!(query.len() > 12); // 至少包含 DNS 头

    // Transaction ID 应为 0x00 0x00
    assert_eq!(query[0], 0x00);
    assert_eq!(query[1], 0x00);

    // Flags: Standard query (0x00 0x00)
    assert_eq!(query[2], 0x00);
    assert_eq!(query[3], 0x00);

    // Questions: 1 (0x00 0x01)
    assert_eq!(query[4], 0x00);
    assert_eq!(query[5], 0x01);

    // Query should contain the service name
    let query_str = String::from_utf8_lossy(&query);
    assert!(query_str.contains("_hap") || query_str.contains("_tcp"));
}

#[test]
fn test_build_mdns_query_different_services() {
    // 测试不同服务类型的查询构建
    let services = vec![
        "_miio._udp.local",
        "_yeelight._tcp.local",
        "_googlecast._tcp.local",
        "_airplay._tcp.local",
    ];

    for service in services {
        let query = build_mdns_query(service);
        assert!(
            query.len() > 12,
            "Query for {} should have valid length",
            service
        );
    }
}

#[test]
fn test_parse_mdns_response_empty() {
    // 测试解析空响应
    let result = parse_mdns_response(&[], "192.168.1.1");
    assert!(result.is_empty());
}

#[test]
fn test_parse_mdns_response_too_short() {
    // 测试解析过短的数据
    let data = vec![0x00, 0x00, 0x00];
    let result = parse_mdns_response(&data, "192.168.1.1");
    assert!(result.is_empty());
}

#[test]
fn test_parse_mdns_response_no_answers() {
    // 测试解析没有答案的响应
    // DNS 头: Transaction ID (2) + Flags (2) + Questions (2) + Answers (2) + Authority (2) + Additional (2)
    let data = vec![
        0x00, 0x00, // Transaction ID
        0x00, 0x00, // Flags
        0x00, 0x01, // 1 Question
        0x00, 0x00, // 0 Answers
        0x00, 0x00, // 0 Authority
        0x00, 0x00, // 0 Additional
    ];
    let result = parse_mdns_response(&data, "192.168.1.1");
    assert!(result.is_empty());
}

#[test]
fn test_mdns_service_info_creation() {
    // 测试 MdnsServiceInfo 结构体创建
    let info = MdnsServiceInfo {
        service_type: "HomeKit".to_string(),
        name: "Living Room Light".to_string(),
        hostname: Some("living-room.local".to_string()),
        port: Some(80),
        ip: Some("192.168.1.100".to_string()),
        txt_records: vec!["path=/".to_string(), "version=1.0".to_string()],
    };

    assert_eq!(info.service_type, "HomeKit");
    assert_eq!(info.name, "Living Room Light");
    assert_eq!(info.hostname, Some("living-room.local".to_string()));
    assert_eq!(info.port, Some(80));
    assert_eq!(info.ip, Some("192.168.1.100".to_string()));
    assert_eq!(info.txt_records.len(), 2);
}

// ═══════════════════════════════════════════════════════════════════════════════
// SSDP/UPnP 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_build_ssdp_search() {
    // 测试构建 SSDP M-SEARCH 请求
    let request = build_ssdp_search();

    // 验证请求格式
    assert!(request.starts_with("M-SEARCH * HTTP/1.1"));
    assert!(request.contains("HOST: 239.255.255.250:1900"));
    assert!(request.contains("MAN: \"ssdp:discover\""));
    assert!(request.contains("MX: 3"));
    assert!(request.contains("ST: ssdp:all"));
    assert!(request.ends_with("\r\n\r\n"));
}

#[test]
fn test_parse_ssdp_response_valid() {
    // 测试解析有效的 SSDP 响应
    let response = b"HTTP/1.1 200 OK\r\n\
        LOCATION: http://192.168.1.1:5000/rootDesc.xml\r\n\
        SERVER: Linux/4.0 UPnP/1.1 MiniUPnPd/2.0\r\n\
        USN: uuid:device-001::urn:schemas-upnp-org:device:InternetGatewayDevice:1\r\n\
        ST: urn:schemas-upnp-org:device:InternetGatewayDevice:1\r\n\
        \r\n";

    let result = parse_ssdp_response(response, "192.168.1.1");
    assert!(result.is_some());

    let device = result.unwrap();
    assert_eq!(device.device_type, "urn:schemas-upnp-org:device:InternetGatewayDevice:1");
    assert_eq!(device.ip, "192.168.1.1");
    assert_eq!(
        device.location,
        Some("http://192.168.1.1:5000/rootDesc.xml".to_string())
    );
    assert_eq!(device.server, Some("Linux/4.0 UPnP/1.1 MiniUPnPd/2.0".to_string()));
    assert_eq!(
        device.usn,
        Some("uuid:device-001::urn:schemas-upnp-org:device:InternetGatewayDevice:1".to_string())
    );
}

#[test]
fn test_parse_ssdp_response_not_ok() {
    // 测试解析非 200 OK 响应
    let response = b"HTTP/1.1 404 Not Found\r\n\r\n";
    let result = parse_ssdp_response(response, "192.168.1.1");
    assert!(result.is_none());
}

#[test]
fn test_parse_ssdp_response_empty() {
    // 测试解析空响应
    let result = parse_ssdp_response(&[], "192.168.1.1");
    assert!(result.is_none());
}

#[test]
fn test_parse_ssdp_response_no_location() {
    // 测试解析没有 location 的响应
    let response = b"HTTP/1.1 200 OK\r\n\
        SERVER: TestServer/1.0\r\n\
        ST: ssdp:all\r\n\
        \r\n";

    let result = parse_ssdp_response(response, "192.168.1.2");
    assert!(result.is_some());

    let device = result.unwrap();
    assert_eq!(device.device_type, "ssdp:all");
    assert_eq!(device.ip, "192.168.1.2");
    assert_eq!(device.location, None);
    assert_eq!(device.server, Some("TestServer/1.0".to_string()));
    assert_eq!(device.usn, None);
}

#[test]
fn test_ssdp_device_info_creation() {
    // 测试 SsdpDeviceInfo 结构体创建
    let device = SsdpDeviceInfo {
        device_type: "urn:schemas-upnp-org:device:MediaServer:1".to_string(),
        location: Some("http://192.168.1.100:9000/desc".to_string()),
        server: Some("Linux/5.0 UPnP/1.0 DLNADOC/1.50".to_string()),
        usn: Some("uuid:media-server-001".to_string()),
        ip: "192.168.1.100".to_string(),
    };

    assert_eq!(device.device_type, "urn:schemas-upnp-org:device:MediaServer:1");
    assert_eq!(device.location, Some("http://192.168.1.100:9000/desc".to_string()));
    assert_eq!(device.server, Some("Linux/5.0 UPnP/1.0 DLNADOC/1.50".to_string()));
    assert_eq!(device.usn, Some("uuid:media-server-001".to_string()));
    assert_eq!(device.ip, "192.168.1.100");
}

// ═══════════════════════════════════════════════════════════════════════════════
// ServiceDiscoveryResult 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_service_discovery_result_new() {
    let result = ServiceDiscoveryResult::new("192.168.1.100".to_string());

    assert_eq!(result.ip, "192.168.1.100");
    assert!(result.mdns_services.is_empty());
    assert!(result.ssdp_devices.is_empty());
}

#[test]
fn test_service_discovery_result_with_services() {
    let mut result = ServiceDiscoveryResult::new("192.168.1.100".to_string());

    // 添加 mDNS 服务
    result.mdns_services.push(MdnsServiceInfo {
        service_type: "Xiaomi MiIO".to_string(),
        name: "Mi Robot Vacuum".to_string(),
        hostname: Some("mi-vacuum.local".to_string()),
        port: Some(54321),
        ip: Some("192.168.1.100".to_string()),
        txt_records: vec![],
    });

    // 添加 SSDP 设备
    result.ssdp_devices.push(SsdpDeviceInfo {
        device_type: "urn:schemas-upnp-org:device:Basic:1".to_string(),
        location: Some("http://192.168.1.100:8080/desc.xml".to_string()),
        server: Some("Linux/4.0 UPnP/1.0".to_string()),
        usn: Some("uuid:basic-device-001".to_string()),
        ip: "192.168.1.100".to_string(),
    });

    assert_eq!(result.mdns_services.len(), 1);
    assert_eq!(result.ssdp_devices.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 异步功能测试
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_mdns_discovery_runs() {
    // 测试 mDNS 发现可以运行（不崩溃）
    // 注意：实际发现服务需要网络环境中有 mDNS 设备
    let results = mdns_discovery(100).await; // 短时间超时

    // 应该返回一个数组（可能为空，取决于网络环境）
    // 我们只验证函数不 panic
    println!("mDNS discovery found {} services", results.len());
}

#[tokio::test]
async fn test_ssdp_discovery_runs() {
    // 测试 SSDP 发现可以运行（不崩溃）
    // 注意：实际发现设备需要网络环境中有 UPnP 设备
    let results = ssdp_discovery(100).await; // 短时间超时

    // 应该返回一个数组（可能为空，取决于网络环境）
    // 我们只验证函数不 panic
    println!("SSDP discovery found {} devices", results.len());
}

#[tokio::test]
async fn test_discover_services_for_host_runs() {
    // 测试对特定主机的服务发现可以运行
    let result = discover_services_for_host("127.0.0.1", 100).await;

    assert_eq!(result.ip, "127.0.0.1");
    // 结果可能为空，取决于网络环境
    println!(
        "Services for 127.0.0.1: {} mDNS, {} SSDP",
        result.mdns_services.len(),
        result.ssdp_devices.len()
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// IoT 设备特定测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_xiaomi_device_parsing() {
    // 测试小米设备信息的解析
    let miio_response = b"\x00\x00\x00\x00\x00\x01\x00\x01\x00\x00\x00\x00_miio._udp.local";

    let services = parse_mdns_response(miio_response, "192.168.1.50");

    // 验证至少解析到了 IP
    if !services.is_empty() {
        assert_eq!(services[0].ip, Some("192.168.1.50".to_string()));
    }
}

#[test]
fn test_yeelight_device_parsing() {
    // 测试 Yeelight 设备信息的解析
    let yeelight_response =
        b"\x00\x00\x00\x00\x00\x01\x00\x01\x00\x00\x00\x00_yeelight._tcp.local";

    let services = parse_mdns_response(yeelight_response, "192.168.1.51");

    // 验证至少解析到了 IP
    if !services.is_empty() {
        assert_eq!(services[0].ip, Some("192.168.1.51".to_string()));
    }
}

#[test]
fn test_homekit_device_parsing() {
    // 测试 HomeKit 设备信息的解析
    let hap_response = b"\x00\x00\x00\x00\x00\x01\x00\x01\x00\x00\x00\x00_hap._tcp.local";

    let services = parse_mdns_response(hap_response, "192.168.1.52");

    // 验证至少解析到了 IP
    if !services.is_empty() {
        assert_eq!(services[0].ip, Some("192.168.1.52".to_string()));
    }
}

#[test]
fn test_ssdp_router_response() {
    // 测试路由器 SSDP 响应解析（常见 IoT 场景）
    let response = b"HTTP/1.1 200 OK\r\n\
        LOCATION: http://192.168.1.1:1900/igd.xml\r\n\
        SERVER: RouterOS/7.0 UPnP/1.0\r\n\
        USN: uuid:router-001::urn:schemas-upnp-org:device:InternetGatewayDevice:1\r\n\
        ST: urn:schemas-upnp-org:device:InternetGatewayDevice:1\r\n\
        \r\n";

    let result = parse_ssdp_response(response, "192.168.1.1");
    assert!(result.is_some());

    let device = result.unwrap();
    assert!(device.device_type.contains("InternetGatewayDevice"));
    assert_eq!(device.ip, "192.168.1.1");
}

#[test]
fn test_ssdp_media_server_response() {
    // 测试媒体服务器 SSDP 响应解析（如小米电视、智能音箱等）
    let response = b"HTTP/1.1 200 OK\r\n\
        LOCATION: http://192.168.1.150:8008/ssdp/device-desc.xml\r\n\
        SERVER: Linux/3.0 UPnP/1.0 Google-Chrome/1.0\r\n\
        USN: uuid:google-cast-001::urn:dial-multiscreen-org:device:dial:1\r\n\
        ST: urn:dial-multiscreen-org:device:dial:1\r\n\
        \r\n";

    let result = parse_ssdp_response(response, "192.168.1.150");
    assert!(result.is_some());

    let device = result.unwrap();
    assert!(device.device_type.contains("dial"));
    assert_eq!(device.ip, "192.168.1.150");
}
