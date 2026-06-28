use crate::security;
use std::net::{SocketAddr, SocketAddrV4};

#[test]
fn test_safe_compare() {
    assert!(security::safe_compare("1234", "1234"));
    assert!(!security::safe_compare("1234", "5678"));
    assert!(!security::safe_compare("1234", "12345"));
}

#[test]
fn test_lockout_attempts() {
    let ip = "127.0.0.1";
    security::reset_attempts(ip);
    assert!(!security::is_locked_out(ip));

    for _ in 0..5 {
        let _ = security::record_attempt(ip);
    }
    assert!(security::is_locked_out(ip));

    security::reset_attempts(ip);
    assert!(!security::is_locked_out(ip));
}

#[test]
fn test_get_client_ip_ignores_xff_without_trusted_list() {
    use std::net::Ipv4Addr;
    let mut headers = axum::http::HeaderMap::new();
    headers.insert("x-forwarded-for", "203.0.113.5".parse().unwrap());
    let socket = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(10, 0, 0, 1), 4401));
    let ip = security::get_client_ip(
        &headers, socket, true, None,
    );
    assert_eq!(ip, "10.0.0.1");
}

#[test]
fn test_get_client_ip_honors_xff_when_trusted_list_matches() {
    use std::net::Ipv4Addr;
    let mut headers = axum::http::HeaderMap::new();
    headers.insert("x-forwarded-for", "203.0.113.5".parse().unwrap());
    let socket = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(10, 0, 0, 1), 4401));
    let trusted = vec!["10.0.0.0/8".to_string()];
    let ip = security::get_client_ip(&headers, socket, true, Some(&trusted));
    assert_eq!(ip, "203.0.113.5");
}

#[test]
fn test_get_client_ip_ignores_xff_when_socket_not_in_trusted_list() {
    use std::net::Ipv4Addr;
    let mut headers = axum::http::HeaderMap::new();
    headers.insert("x-forwarded-for", "203.0.113.5".parse().unwrap());
    let socket = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(192, 168, 1, 1), 4401));
    let trusted = vec!["10.0.0.0/8".to_string()];
    let ip = security::get_client_ip(&headers, socket, true, Some(&trusted));
    assert_eq!(ip, "192.168.1.1");
}

#[test]
fn test_get_client_ip_ignores_xff_when_trust_proxy_false() {
    use std::net::Ipv4Addr;
    let mut headers = axum::http::HeaderMap::new();
    headers.insert("x-forwarded-for", "203.0.113.5".parse().unwrap());
    let socket = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(10, 0, 0, 1), 4401));
    let ip = security::get_client_ip(&headers, socket, false, None);
    assert_eq!(ip, "10.0.0.1");
}
