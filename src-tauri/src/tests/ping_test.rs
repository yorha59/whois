//! ICMP Ping 主机存活检测测试

use crate::{ping_host, ping_host_with_retries};

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 1.3: ICMP Ping 基础测试
// ═══════════════════════════════════════════════════════════════════════════════

/// 测试 ping localhost（应该成功）
#[tokio::test]
async fn test_ping_localhost() {
    // 127.0.0.1 应该总是响应 ping
    let result = ping_host("127.0.0.1").await;
    // 在某些环境中可能需要 root 权限才能发送 ICMP
    // 所以这个测试主要验证函数不会 panic
    println!("Ping 127.0.0.1: {}", result);
}

/// 测试 ping 无效 IP 应该返回 false
#[tokio::test]
async fn test_ping_invalid_ip() {
    let result = ping_host("invalid_ip").await;
    assert!(!result, "Invalid IP should return false");
}

/// 测试 ping 空字符串应该返回 false
#[tokio::test]
async fn test_ping_empty_string() {
    let result = ping_host("").await;
    assert!(!result, "Empty string should return false");
}

/// 测试 ping 格式错误的 IP 应该返回 false
#[tokio::test]
async fn test_ping_malformed_ip() {
    let malformed_ips = vec![
        "192.168.1",       // 缺少一个八位组
        "192.168.1.1.1",   // 多了一个八位组
        "192.168.1.256",   // 数值超出范围
        "192.168.1.-1",    // 负数
        "abc.def.ghi.jkl", // 非数字
        "...",             // 只有点
    ];

    for ip in malformed_ips {
        let result = ping_host(ip).await;
        assert!(!result, "Malformed IP '{}' should return false", ip);
    }
}

/// 测试 ping IPv6 地址（当前暂不支持，应该返回 false）
#[tokio::test]
async fn test_ping_ipv6() {
    let result = ping_host("::1").await;
    // IPv6 暂不支持，应该返回 false
    assert!(!result, "IPv6 should return false (not supported yet)");

    let result = ping_host("2001:db8::1").await;
    assert!(!result, "IPv6 should return false (not supported yet)");
}

/// 测试 ping 私有网段格式 IP
#[tokio::test]
async fn test_ping_private_ips() {
    // 这些只是格式测试，实际上 ping 可能超时
    // 主要验证函数能正确解析这些 IP 格式
    let private_ips = vec![
        "192.168.1.1",
        "10.0.0.1",
        "172.16.0.1",
        "172.31.255.255",
    ];

    for ip in private_ips {
        // 这些 IP 格式是正确的，但由于超时或不可达，可能返回 true 或 false
        // 主要验证函数不会 panic
        let _result = ping_host(ip).await;
        println!("Ping {}: {}", ip, _result);
    }
}

/// 测试 ping 边界情况 IP
#[tokio::test]
async fn test_ping_edge_case_ips() {
    let edge_cases = vec![
        "0.0.0.0",
        "0.0.0.1",
        "255.255.255.255",
        "255.255.255.0",
    ];

    for ip in edge_cases {
        // 这些 IP 格式正确，主要验证不会 panic
        let _result = ping_host(ip).await;
        println!("Ping {}: {}", ip, _result);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 1.3: 并发 ping 测试
// ═══════════════════════════════════════════════════════════════════════════════

/// 测试并发 ping 多个主机
#[tokio::test]
async fn test_ping_concurrent() {
    let ips = vec![
        "127.0.0.1",
        "192.168.1.1",
        "192.168.1.2",
        "192.168.1.3",
    ];

    let mut handles = vec![];
    for ip in ips {
        handles.push(tokio::spawn(async move {
            let result = ping_host(ip).await;
            (ip, result)
        }));
    }

    for handle in handles {
        let (ip, result) = handle.await.unwrap();
        println!("Ping {}: {}", ip, result);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 1.3: 超时测试
// ═══════════════════════════════════════════════════════════════════════════════

/// 测试 ping 超时行为（ping 不存在的 IP 应该快速返回 false）
#[tokio::test]
async fn test_ping_timeout() {
    use tokio::time::{timeout, Duration};

    // 使用一个不太可能响应的 IP
    let start = std::time::Instant::now();
    let result = timeout(Duration::from_secs(2), ping_host("192.0.2.1")).await;
    let elapsed = start.elapsed();

    // 无论结果如何，应该在合理时间内返回（< 2 秒）
    assert!(
        elapsed < Duration::from_secs(2),
        "Ping should return quickly due to 500ms timeout"
    );

    println!("Ping timeout test completed in {:?}, result: {:?}", elapsed, result);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 1.3: 带重试的 ping 测试
// ═══════════════════════════════════════════════════════════════════════════════

/// 测试带重试的 ping localhost（应该成功）
#[tokio::test]
async fn test_ping_with_retries_localhost() {
    // 127.0.0.1 应该总是响应 ping，使用重试机制
    let result = ping_host_with_retries("127.0.0.1", 500, 2).await;
    println!("Ping with retries 127.0.0.1: {}", result);
}

/// 测试带重试的 ping 无效 IP
#[tokio::test]
async fn test_ping_with_retries_invalid_ip() {
    let result = ping_host_with_retries("invalid_ip", 500, 2).await;
    assert!(!result, "Invalid IP should return false even with retries");
}

/// 测试带重试的 ping IPv6（暂不支持）
#[tokio::test]
async fn test_ping_with_retries_ipv6() {
    let result = ping_host_with_retries("::1", 500, 2).await;
    assert!(!result, "IPv6 should return false (not supported yet)");
}

/// 测试 ping 重试机制的超时行为
#[tokio::test]
async fn test_ping_with_retries_timeout() {
    use tokio::time::{timeout, Duration};

    // 使用一个不太可能响应的 IP，设置较短的超时和较少的重试
    let start = std::time::Instant::now();
    let result = timeout(
        Duration::from_secs(5),
        ping_host_with_retries("192.0.2.1", 300, 1) // 300ms 超时，1 次重试
    ).await;
    let elapsed = start.elapsed();

    // 应该快速返回（300ms × 2 次尝试 + 50ms 延迟 ≈ 650ms）
    assert!(
        elapsed < Duration::from_secs(2),
        "Ping with retries should return quickly: elapsed {:?}",
        elapsed
    );

    println!("Ping with retries timeout test completed in {:?}, result: {:?}", elapsed, result);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 1.3: 集成测试（需要实际网络环境）
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// 集成测试：ping 实际的主机
    /// 默认忽略，需要手动运行: cargo test --test integration -- --ignored
    #[tokio::test]
    #[ignore]
    async fn test_ping_real_hosts() {
        // 测试一些公共 DNS 服务器
        let public_dns = vec![
            "8.8.8.8",    // Google DNS
            "1.1.1.1",    // Cloudflare DNS
            "114.114.114.114", // 114 DNS
        ];

        for ip in public_dns {
            let result = ping_host(ip).await;
            println!("Ping {}: {}", ip, result);
            // 如果网络正常，这些应该返回 true
            // 但由于网络环境不同，不做强制断言
        }
    }

    /// 集成测试：使用重试机制 ping 实际的主机
    #[tokio::test]
    #[ignore]
    async fn test_ping_with_retries_real_hosts() {
        let public_dns = vec![
            "8.8.8.8",
            "1.1.1.1",
        ];

        for ip in public_dns {
            // 使用更短的超时但更多重试
            let result = ping_host_with_retries(ip, 300, 3).await;
            println!("Ping with retries {}: {}", ip, result);
        }
    }
}
