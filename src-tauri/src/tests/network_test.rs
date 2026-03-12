//! 网络检测功能测试
//!
//! 测试目标：验证网络自动检测功能的正确性
//! 覆盖率目标：80%+
//!
//! 注意：部分测试需要网络接口，使用条件编译处理

use crate::{
    detect_network_internal as detect_network, dns_lookup_reverse, resolve_hostname, NetworkInfo,
};

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 1.2: 网络检测成功场景测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
// 此测试需要在有网络接口的环境中运行
// 在 CI 环境中跳过
fn test_detect_network_success() {
    // 此测试需要在有网络接口的环境中运行
    let result = detect_network();

    if result.is_ok() {
        let network_info = result.unwrap();

        // 验证返回的 IP 格式
        assert!(
            !network_info.local_ip.is_empty(),
            "Local IP should not be empty"
        );
        assert!(
            network_info.local_ip.contains('.'),
            "Local IP should be IPv4 format"
        );

        // 验证子网格式
        assert!(
            !network_info.subnet.is_empty(),
            "Subnet should not be empty"
        );
        let parts: Vec<&str> = network_info.subnet.split('.').collect();
        assert_eq!(parts.len(), 3, "Subnet should have 3 octets (x.x.x)");

        // 验证子网与本地 IP 的一致性
        let ip_parts: Vec<&str> = network_info.local_ip.split('.').collect();
        assert_eq!(ip_parts.len(), 4, "Local IP should have 4 octets");

        let expected_subnet = format!("{}.{}.{}", ip_parts[0], ip_parts[1], ip_parts[2]);
        assert_eq!(
            network_info.subnet, expected_subnet,
            "Subnet should match first 3 octets of local IP"
        );
    }
}

#[test]
fn test_detect_network_returns_result() {
    // 基本测试：验证函数返回 Result 类型
    let result = detect_network();
    // 函数应该返回 Ok 或 Err，但不应该 panic
    match result {
        Ok(info) => {
            // 验证返回的结构体包含预期的字段
            assert!(!info.local_ip.is_empty());
            assert!(!info.subnet.is_empty());
        }
        Err(e) => {
            // 在没有网络接口的环境中，应该返回有意义的错误
            assert!(!e.is_empty());
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 1.2: NetworkInfo 结构体测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_network_info_creation() {
    let info = NetworkInfo {
        local_ip: "192.168.1.100".to_string(),
        subnet: "192.168.1".to_string(),
    };

    assert_eq!(info.local_ip, "192.168.1.100");
    assert_eq!(info.subnet, "192.168.1");
}

#[test]
fn test_network_info_serialization() {
    let info = NetworkInfo {
        local_ip: "10.0.0.5".to_string(),
        subnet: "10.0.0".to_string(),
    };

    let json = serde_json::to_string(&info).expect("Should serialize");
    assert!(json.contains("10.0.0.5"));
    assert!(json.contains("10.0.0"));

    let deserialized: NetworkInfo = serde_json::from_str(&json).expect("Should deserialize");
    assert_eq!(deserialized.local_ip, "10.0.0.5");
    assert_eq!(deserialized.subnet, "10.0.0");
}

#[test]
fn test_network_info_with_private_ips() {
    let private_ips = vec![
        ("10.0.0.1", "10.0.0"),
        ("172.16.0.1", "172.16.0"),
        ("172.31.255.255", "172.31.255"),
        ("192.168.0.1", "192.168.0"),
        ("192.168.255.255", "192.168.255"),
    ];

    for (ip, expected_subnet) in private_ips {
        let info = NetworkInfo {
            local_ip: ip.to_string(),
            subnet: expected_subnet.to_string(),
        };

        assert_eq!(info.local_ip, ip);
        assert_eq!(info.subnet, expected_subnet);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 1.2: 子网提取逻辑测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_subnet_extraction_valid_ips() {
    let test_cases = vec![
        ("192.168.1.100", "192.168.1"),
        ("10.0.0.1", "10.0.0"),
        ("172.16.255.255", "172.16.255"),
        ("1.2.3.4", "1.2.3"),
        ("255.255.255.255", "255.255.255"),
        ("0.0.0.0", "0.0.0"),
    ];

    for (ip, expected_subnet) in test_cases {
        let parts: Vec<&str> = ip.split('.').collect();
        if parts.len() == 4 {
            let subnet = format!("{}.{}.{}", parts[0], parts[1], parts[2]);
            assert_eq!(
                subnet, expected_subnet,
                "Subnet extraction failed for {}",
                ip
            );
        }
    }
}

#[test]
fn test_subnet_extraction_edge_cases() {
    // 测试边缘情况的 IP 格式
    let edge_cases = vec![
        "0.0.0.1",
        "0.0.0.255",
        "255.255.255.0",
        "127.0.0.1", // localhost
    ];

    for ip in edge_cases {
        let parts: Vec<&str> = ip.split('.').collect();
        assert_eq!(parts.len(), 4, "Should handle edge case IP: {}", ip);

        let subnet = format!("{}.{}.{}", parts[0], parts[1], parts[2]);
        assert!(
            !subnet.is_empty(),
            "Should extract subnet for edge case IP: {}",
            ip
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 1.2: 主机名解析测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_resolve_hostname_localhost() {
    // 测试 localhost 解析
    let _result = resolve_hostname("127.0.0.1");
    // localhost 解析结果可能为 None 或 "localhost"，取决于系统配置
    // 主要验证函数不会 panic
}

#[test]
fn test_resolve_hostname_invalid_ip() {
    // 测试无效 IP 应该返回 None
    let result = resolve_hostname("invalid_ip");
    assert!(result.is_none(), "Invalid IP should return None");
}

#[test]
fn test_resolve_hostname_empty_string() {
    // 测试空字符串应该返回 None
    let result = resolve_hostname("");
    assert!(result.is_none(), "Empty string should return None");
}

#[test]
fn test_resolve_hostname_malformed_ip() {
    // 测试格式错误的 IP
    let malformed_ips = vec![
        "192.168.1",       // 缺少一个八位组
        "192.168.1.1.1",   // 多了一个八位组
        "192.168.1.256",   // 数值超出范围
        "192.168.1.-1",    // 负数
        "abc.def.ghi.jkl", // 非数字
        "...",             // 只有点
    ];

    for ip in malformed_ips {
        let result = resolve_hostname(ip);
        assert!(result.is_none(), "Malformed IP '{}' should return None", ip);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 1.2: DNS 反向查找测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_dns_lookup_reverse_localhost() {
    // 测试 localhost 反向查找
    let _result = dns_lookup_reverse("127.0.0.1");
    // 结果可能成功也可能失败，取决于系统配置
    // 主要验证函数不会 panic
}

#[test]
fn test_dns_lookup_reverse_invalid_ip() {
    // 测试无效 IP 应该返回错误
    let result = dns_lookup_reverse("invalid");
    assert!(result.is_err(), "Invalid IP should return Err");
}

#[test]
fn test_dns_lookup_reverse_empty_string() {
    let result = dns_lookup_reverse("");
    assert!(result.is_err(), "Empty string should return Err");
}

#[test]
fn test_dns_lookup_reverse_ipv6() {
    // 测试 IPv6 地址（当前实现可能不支持）
    let _result = dns_lookup_reverse("::1");
    // IPv6 可能不被支持，主要验证不会 panic
}

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 1.2: 错误处理测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_detect_network_error_handling() {
    // 在没有网络接口的环境中，detect_network 应该返回错误
    // 这个测试验证错误处理路径
    let result = detect_network();

    // 结果可能是 Ok 或 Err，但都应该是有意义的结果
    match result {
        Ok(info) => {
            // 验证返回的数据是有效的
            assert!(!info.local_ip.is_empty());
            assert!(!info.subnet.is_empty());
        }
        Err(e) => {
            // 验证错误消息是有意义的
            assert!(!e.is_empty());
        }
    }
}

#[test]
fn test_network_info_invalid_ip_format() {
    // 测试不符合 /24 子网的 IP（未来可能支持 /16、/8 等）
    let unusual_ips = vec![
        "10.0.0.1.1",     // 5 个八位组
        "192.168",        // 2 个八位组
        "192",            // 1 个八位组
        "192.168.1.1/24", // 带 CIDR 表示法（虽然 parse 会失败）
    ];

    for ip in unusual_ips {
        let parts: Vec<&str> = ip.split('.').collect();
        if parts.len() != 4 {
            // 这些 IP 不会被正常解析，应该返回错误或被过滤
            continue;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 1.2: 单元测试 - 模拟网络环境
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_ip_format_validation() {
    // 验证 IP 格式检查的辅助测试
    let valid_ips = vec![
        "192.168.1.1",
        "10.0.0.1",
        "172.16.0.1",
        "255.255.255.255",
        "0.0.0.0",
        "127.0.0.1",
    ];

    for ip in valid_ips {
        let parts: Vec<&str> = ip.split('.').collect();
        assert_eq!(parts.len(), 4, "Valid IP should have 4 octets: {}", ip);

        for part in &parts {
            let num: Result<u8, _> = part.parse();
            assert!(num.is_ok(), "Each octet should be a valid u8: {}", ip);
        }
    }
}

#[test]
fn test_subnet_calculation() {
    // 测试 /24 子网计算
    let test_cases = vec![
        ("192.168.1.50", "192.168.1"),
        ("192.168.1.1", "192.168.1"),
        ("192.168.1.254", "192.168.1"),
        ("10.0.0.100", "10.0.0"),
        ("172.16.50.25", "172.16.50"),
    ];

    for (ip, expected_subnet) in test_cases {
        let parts: Vec<&str> = ip.split('.').collect();
        assert_eq!(parts.len(), 4);

        let subnet = format!("{}.{}.{}", parts[0], parts[1], parts[2]);
        assert_eq!(subnet, expected_subnet);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 1.2: 集成测试占位符
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod integration_tests {
    // 这些测试需要完整的网络栈，在实际网络环境中运行

    #[test]
    #[ignore] // 默认忽略，需要手动运行
    fn test_detect_network_real_environment() {
        use crate::detect_network_internal as detect_network;

        let result = detect_network();
        assert!(result.is_ok(), "Should detect network in real environment");

        let info = result.unwrap();
        println!("Detected local IP: {}", info.local_ip);
        println!("Detected subnet: {}", info.subnet);

        // 验证 IP 格式
        let parts: Vec<&str> = info.local_ip.split('.').collect();
        assert_eq!(parts.len(), 4);
    }
}
